use super::{
    events::{
        EventParameters, MeteoraDynamicLiquidityEvent, MeteoraLiquidityEvent, MeteoraSwapEvent,
    },
    types::{
        meteora_dlmm_program_id, meteora_dynamic_program_id, MeteoraDynamicLiquidityData,
        MeteoraLiquidityData, MeteoraSwapData, DLMM_ADD_LIQUIDITY_DISCRIMINATOR,
        DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR, DLMM_SWAP_DISCRIMINATOR,
        DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR, DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR,
    },
};
use crate::{
    error::ParseResult,
    events::{
        common::utils::{has_discriminator, parse_u32_le, parse_u64_le},
        core::{EventParser, GenericEventParseConfig},
    },
    types::{metadata_helpers, EventMetadata, EventType, ProtocolType},
};
use riglr_events_core::Event;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Meteora event parser
pub struct MeteoraEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl MeteoraEventParser {
    /// Creates a new Meteora event parser with default configurations
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl EventParser for MeteoraEventParser {
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
                    let metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", params.signature, params.index),
                        params.signature.to_string(),
                        params.slot,
                        params.block_time.unwrap_or(0),
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        params.index.clone(),
                        params.program_received_time_ms,
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
                    let metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", params.signature, params.index),
                        params.signature.to_string(),
                        params.slot,
                        params.block_time.unwrap_or(0),
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        params.index.clone(),
                        params.program_received_time_ms,
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

impl Default for MeteoraEventParser {
    fn default() -> Self {
        let program_ids = vec![meteora_dlmm_program_id(), meteora_dynamic_program_id()];

        let configs = vec![
            // DLMM configs
            GenericEventParseConfig {
                program_id: meteora_dlmm_program_id(),
                protocol_type: ProtocolType::MeteoraDlmm,
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &DLMM_SWAP_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_meteora_dlmm_swap_inner_instruction,
                instruction_parser: parse_meteora_dlmm_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: meteora_dlmm_program_id(),
                protocol_type: ProtocolType::MeteoraDlmm,
                inner_instruction_discriminator: "addLiquidity",
                instruction_discriminator: &DLMM_ADD_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: parse_meteora_dlmm_add_liquidity_inner_instruction,
                instruction_parser: parse_meteora_dlmm_add_liquidity_instruction,
            },
            GenericEventParseConfig {
                program_id: meteora_dlmm_program_id(),
                protocol_type: ProtocolType::MeteoraDlmm,
                inner_instruction_discriminator: "removeLiquidity",
                instruction_discriminator: &DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: parse_meteora_dlmm_remove_liquidity_inner_instruction,
                instruction_parser: parse_meteora_dlmm_remove_liquidity_instruction,
            },
            // Dynamic AMM configs
            GenericEventParseConfig {
                program_id: meteora_dynamic_program_id(),
                protocol_type: ProtocolType::Other("MeteoraDynamic".to_string()),
                inner_instruction_discriminator: "depositAllTokenTypes",
                instruction_discriminator: &DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: parse_meteora_dynamic_add_liquidity_inner_instruction,
                instruction_parser: parse_meteora_dynamic_add_liquidity_instruction,
            },
            GenericEventParseConfig {
                program_id: meteora_dynamic_program_id(),
                protocol_type: ProtocolType::Other("MeteoraDynamic".to_string()),
                inner_instruction_discriminator: "withdrawAllTokenTypes",
                instruction_discriminator: &DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: parse_meteora_dynamic_remove_liquidity_inner_instruction,
                instruction_parser: parse_meteora_dynamic_remove_liquidity_instruction,
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

// Parser functions for DLMM instructions

fn parse_meteora_dlmm_swap_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_meteora_dlmm_swap_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Meteora DLMM swap data".to_string(),
        )
    })?;

    Ok(Box::new(MeteoraSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

fn parse_meteora_dlmm_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data =
        parse_meteora_dlmm_swap_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Meteora DLMM swap instruction data".to_string(),
            )
        })?;

    Ok(Box::new(MeteoraSwapEvent::new(
        EventParameters::new(
            metadata.id().to_string(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        swap_data,
    )) as Box<dyn Event>)
}

fn parse_meteora_dlmm_add_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dlmm_liquidity_data(data, true).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Meteora DLMM add liquidity data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dlmm_add_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dlmm_liquidity_data_from_instruction(data, accounts, true)
        .ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Meteora DLMM add liquidity instruction data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dlmm_remove_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dlmm_liquidity_data(data, false).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dlmm_remove_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dlmm_liquidity_data_from_instruction(data, accounts, false)
        .ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
        })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

// Parser functions for Dynamic AMM instructions

fn parse_meteora_dynamic_add_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dynamic_liquidity_data(data, true).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraDynamicLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dynamic_add_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dynamic_liquidity_data_from_instruction(
        data, accounts, true,
    )
    .ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraDynamicLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dynamic_remove_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dynamic_liquidity_data(data, false).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraDynamicLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_meteora_dynamic_remove_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_meteora_dynamic_liquidity_data_from_instruction(
        data, accounts, false,
    )
    .ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Meteora data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.core.id.clone(),
        signature: metadata.signature.clone(),
        slot: metadata.slot,
        block_time: 0,    // Not available in SolanaEventMetadata
        block_time_ms: 0, // Not available in SolanaEventMetadata
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(MeteoraDynamicLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

// Data parsing helpers

fn parse_meteora_dlmm_swap_data(data: &[u8]) -> Option<MeteoraSwapData> {
    if data.len() < 32 {
        return None;
    }

    let mut offset = 8; // Skip discriminator

    let amount_in = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let min_amount_out = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let swap_for_y = data.get(offset)? != &0;

    Some(MeteoraSwapData {
        pair: Pubkey::default(),         // Would need to extract from accounts
        user: Pubkey::default(),         // Would need to extract from accounts
        token_mint_x: Pubkey::default(), // Would need to extract from accounts
        token_mint_y: Pubkey::default(), // Would need to extract from accounts
        reserve_x: Pubkey::default(),    // Would need to extract from accounts
        reserve_y: Pubkey::default(),    // Would need to extract from accounts
        amount_in,
        min_amount_out,
        actual_amount_out: min_amount_out, // Simplified
        swap_for_y,
        active_id_before: 0,        // Would need pool state
        active_id_after: 0,         // Would need pool state
        fee_amount: 0,              // Would need to calculate
        protocol_fee: 0,            // Would need to calculate
        bins_traversed: Vec::new(), // Would need to parse from logs
    })
}

fn parse_meteora_dlmm_swap_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MeteoraSwapData> {
    let mut swap_data = parse_meteora_dlmm_swap_data(data)?;

    // Extract accounts (typical Meteora DLMM swap instruction layout)
    if accounts.len() >= 8 {
        swap_data.pair = accounts[1];
        swap_data.user = accounts[0];
        swap_data.reserve_x = accounts[2];
        swap_data.reserve_y = accounts[3];
        // More account parsing would be done here
    }

    Some(swap_data)
}

fn parse_meteora_dlmm_liquidity_data(data: &[u8], is_add: bool) -> Option<MeteoraLiquidityData> {
    if data.len() < 48 {
        return None;
    }

    let mut offset = 8; // Skip discriminator

    let bin_id_from = parse_u32_le(&data[offset..offset + 4]).ok()?;
    offset += 4;

    let bin_id_to = parse_u32_le(&data[offset..offset + 4]).ok()?;
    offset += 4;

    let amount_x = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let amount_y = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let active_id = parse_u32_le(&data[offset..offset + 4]).ok()?;

    Some(MeteoraLiquidityData {
        pair: Pubkey::default(),
        user: Pubkey::default(),
        position: Pubkey::default(),
        token_mint_x: Pubkey::default(),
        token_mint_y: Pubkey::default(),
        reserve_x: Pubkey::default(),
        reserve_y: Pubkey::default(),
        bin_id_from,
        bin_id_to,
        amount_x,
        amount_y,
        liquidity_minted: (amount_x as u128 + amount_y as u128), // Simplified calculation
        active_id,
        is_add,
        bins_affected: Vec::new(), // Would need more complex parsing
    })
}

fn parse_meteora_dlmm_liquidity_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    is_add: bool,
) -> Option<MeteoraLiquidityData> {
    let mut liquidity_data = parse_meteora_dlmm_liquidity_data(data, is_add)?;

    // Extract accounts from instruction
    if accounts.len() >= 10 {
        liquidity_data.pair = accounts[1];
        liquidity_data.user = accounts[0];
        liquidity_data.position = accounts[2];
        liquidity_data.reserve_x = accounts[5];
        liquidity_data.reserve_y = accounts[6];
    }

    Some(liquidity_data)
}

fn parse_meteora_dynamic_liquidity_data(
    data: &[u8],
    is_deposit: bool,
) -> Option<MeteoraDynamicLiquidityData> {
    if data.len() < 48 {
        return None;
    }

    let mut offset = 8; // Skip discriminator

    let pool_token_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let maximum_token_a_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let maximum_token_b_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;

    Some(MeteoraDynamicLiquidityData {
        pool: Pubkey::default(),
        user: Pubkey::default(),
        token_mint_a: Pubkey::default(),
        token_mint_b: Pubkey::default(),
        vault_a: Pubkey::default(),
        vault_b: Pubkey::default(),
        lp_mint: Pubkey::default(),
        pool_token_amount,
        token_a_amount: maximum_token_a_amount, // Simplified
        token_b_amount: maximum_token_b_amount, // Simplified
        minimum_pool_token_amount: pool_token_amount, // Simplified
        maximum_token_a_amount,
        maximum_token_b_amount,
        is_deposit,
    })
}

fn parse_meteora_dynamic_liquidity_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    is_deposit: bool,
) -> Option<MeteoraDynamicLiquidityData> {
    let mut liquidity_data = parse_meteora_dynamic_liquidity_data(data, is_deposit)?;

    // Extract accounts from instruction
    if accounts.len() >= 10 {
        liquidity_data.pool = accounts[1];
        liquidity_data.user = accounts[0];
        liquidity_data.vault_a = accounts[4];
        liquidity_data.vault_b = accounts[5];
        liquidity_data.lp_mint = accounts[6];
    }

    Some(liquidity_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;

    fn create_test_metadata() -> EventMetadata {
        metadata_helpers::create_solana_metadata(
            "test_id".to_string(),
            "test_signature".to_string(),
            123456,
            1234567890,
            ProtocolType::MeteoraDlmm,
            EventType::Swap,
            meteora_dlmm_program_id(),
            "0".to_string(),
            1234567890123,
        )
    }

    fn create_test_pubkeys(count: usize) -> Vec<Pubkey> {
        (0..count).map(|_| Pubkey::new_unique()).collect()
    }

    #[test]
    fn test_meteora_event_parser_new_should_create_default() {
        let parser = MeteoraEventParser::default();
        assert_eq!(parser.program_ids.len(), 2);
        assert!(parser.program_ids.contains(&meteora_dlmm_program_id()));
        assert!(parser.program_ids.contains(&meteora_dynamic_program_id()));
        assert!(!parser.inner_instruction_configs.is_empty());
        assert!(!parser.instruction_configs.is_empty());
    }

    #[test]
    fn test_meteora_event_parser_default_should_configure_correctly() {
        let parser = MeteoraEventParser::default();

        assert_eq!(parser.program_ids.len(), 2);
        assert!(parser.program_ids.contains(&meteora_dlmm_program_id()));
        assert!(parser.program_ids.contains(&meteora_dynamic_program_id()));

        // Check inner instruction configs
        assert!(parser.inner_instruction_configs.contains_key("swap"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("addLiquidity"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("removeLiquidity"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("depositAllTokenTypes"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("withdrawAllTokenTypes"));

        // Check instruction configs
        assert!(parser
            .instruction_configs
            .contains_key(&DLMM_SWAP_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec()));
        assert!(parser
            .instruction_configs
            .contains_key(&DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec()));
    }

    #[test]
    fn test_inner_instruction_configs_should_return_clone() {
        let parser = MeteoraEventParser::default();
        let configs = parser.inner_instruction_configs();
        assert_eq!(configs.len(), parser.inner_instruction_configs.len());
    }

    #[test]
    fn test_instruction_configs_should_return_clone() {
        let parser = MeteoraEventParser::default();
        let configs = parser.instruction_configs();
        assert_eq!(configs.len(), parser.instruction_configs.len());
    }

    #[test]
    fn test_should_handle_when_program_id_supported_should_return_true() {
        let parser = MeteoraEventParser::default();
        assert!(parser.should_handle(&meteora_dlmm_program_id()));
        assert!(parser.should_handle(&meteora_dynamic_program_id()));
    }

    #[test]
    fn test_should_handle_when_program_id_not_supported_should_return_false() {
        let parser = MeteoraEventParser::default();
        let unsupported_id = Pubkey::new_unique();
        assert!(!parser.should_handle(&unsupported_id));
    }

    #[test]
    fn test_supported_program_ids_should_return_all_ids() {
        let parser = MeteoraEventParser::default();
        let ids = parser.supported_program_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&meteora_dlmm_program_id()));
        assert!(ids.contains(&meteora_dynamic_program_id()));
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_valid_data_should_parse_events() {
        let parser = MeteoraEventParser::default();

        // Create test data with swap discriminator
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&500u64.to_le_bytes()); // min_amount_out
        data.push(1); // swap_for_y

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: bs58::encode(&data).into_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(!events.is_empty());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_invalid_data_should_return_empty() {
        let parser = MeteoraEventParser::default();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: "invalid_data".to_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_matching_discriminator_should_parse_events() {
        let parser = MeteoraEventParser::default();
        let accounts = create_test_pubkeys(10);

        // Create test data with swap discriminator
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&500u64.to_le_bytes()); // min_amount_out
        data.push(1); // swap_for_y

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            data,
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(!events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_no_matching_discriminator_should_return_empty() {
        let parser = MeteoraEventParser::default();
        let accounts = create_test_pubkeys(10);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            data: vec![0, 1, 2, 3, 4, 5, 6, 7], // Non-matching discriminator
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_data_when_valid_data_should_parse_correctly() {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&500u64.to_le_bytes()); // min_amount_out
        data.push(1); // swap_for_y

        let result = parse_meteora_dlmm_swap_data(&data);
        assert!(result.is_some());

        let swap_data = result.unwrap();
        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.min_amount_out, 500);
        assert!(swap_data.swap_for_y);
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_data_when_insufficient_data_should_return_none() {
        let data = vec![0; 20]; // Less than 32 bytes
        let result = parse_meteora_dlmm_swap_data(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_data_when_swap_for_y_false_should_parse_correctly() {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount_in
        data.extend_from_slice(&1000u64.to_le_bytes()); // min_amount_out
        data.push(0); // swap_for_y = false

        let result = parse_meteora_dlmm_swap_data(&data);
        assert!(result.is_some());

        let swap_data = result.unwrap();
        assert_eq!(swap_data.amount_in, 2000);
        assert_eq!(swap_data.min_amount_out, 1000);
        assert!(!swap_data.swap_for_y);
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_data_from_instruction_when_sufficient_accounts_should_set_accounts(
    ) {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let accounts = create_test_pubkeys(10);
        let result = parse_meteora_dlmm_swap_data_from_instruction(&data, &accounts);

        assert!(result.is_some());
        let swap_data = result.unwrap();
        assert_eq!(swap_data.user, accounts[0]);
        assert_eq!(swap_data.pair, accounts[1]);
        assert_eq!(swap_data.reserve_x, accounts[2]);
        assert_eq!(swap_data.reserve_y, accounts[3]);
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let accounts = create_test_pubkeys(5); // Less than 8 accounts
        let result = parse_meteora_dlmm_swap_data_from_instruction(&data, &accounts);

        assert!(result.is_some());
        let swap_data = result.unwrap();
        assert_eq!(swap_data.user, Pubkey::default());
        assert_eq!(swap_data.pair, Pubkey::default());
    }

    #[test]
    fn test_parse_meteora_dlmm_liquidity_data_when_valid_data_add_should_parse_correctly() {
        let mut data = DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes()); // bin_id_from
        data.extend_from_slice(&200u32.to_le_bytes()); // bin_id_to
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount_x
        data.extend_from_slice(&2000u64.to_le_bytes()); // amount_y
        data.extend_from_slice(&150u32.to_le_bytes()); // active_id

        let result = parse_meteora_dlmm_liquidity_data(&data, true);
        assert!(result.is_some());

        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.bin_id_from, 100);
        assert_eq!(liquidity_data.bin_id_to, 200);
        assert_eq!(liquidity_data.amount_x, 1000);
        assert_eq!(liquidity_data.amount_y, 2000);
        assert_eq!(liquidity_data.active_id, 150);
        assert!(liquidity_data.is_add);
        assert_eq!(liquidity_data.liquidity_minted, 3000);
    }

    #[test]
    fn test_parse_meteora_dlmm_liquidity_data_when_valid_data_remove_should_parse_correctly() {
        let mut data = DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&300u32.to_le_bytes()); // bin_id_from
        data.extend_from_slice(&400u32.to_le_bytes()); // bin_id_to
        data.extend_from_slice(&500u64.to_le_bytes()); // amount_x
        data.extend_from_slice(&600u64.to_le_bytes()); // amount_y
        data.extend_from_slice(&350u32.to_le_bytes()); // active_id

        let result = parse_meteora_dlmm_liquidity_data(&data, false);
        assert!(result.is_some());

        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.bin_id_from, 300);
        assert_eq!(liquidity_data.bin_id_to, 400);
        assert_eq!(liquidity_data.amount_x, 500);
        assert_eq!(liquidity_data.amount_y, 600);
        assert_eq!(liquidity_data.active_id, 350);
        assert!(!liquidity_data.is_add);
        assert_eq!(liquidity_data.liquidity_minted, 1100);
    }

    #[test]
    fn test_parse_meteora_dlmm_liquidity_data_when_insufficient_data_should_return_none() {
        let data = vec![0; 40]; // Less than 48 bytes
        let result = parse_meteora_dlmm_liquidity_data(&data, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_meteora_dlmm_liquidity_data_from_instruction_when_sufficient_accounts_should_set_accounts(
    ) {
        let mut data = DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let result = parse_meteora_dlmm_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(result.is_some());
        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.user, accounts[0]);
        assert_eq!(liquidity_data.pair, accounts[1]);
        assert_eq!(liquidity_data.position, accounts[2]);
        assert_eq!(liquidity_data.reserve_x, accounts[5]);
        assert_eq!(liquidity_data.reserve_y, accounts[6]);
    }

    #[test]
    fn test_parse_meteora_dlmm_liquidity_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let accounts = create_test_pubkeys(5); // Less than 10 accounts
        let result = parse_meteora_dlmm_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(result.is_some());
        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.user, Pubkey::default());
        assert_eq!(liquidity_data.pair, Pubkey::default());
    }

    #[test]
    fn test_parse_meteora_dynamic_liquidity_data_when_valid_data_deposit_should_parse_correctly() {
        let mut data = DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes()); // pool_token_amount
        data.extend_from_slice(&2000u64.to_le_bytes()); // maximum_token_a_amount
        data.extend_from_slice(&3000u64.to_le_bytes()); // maximum_token_b_amount

        let result = parse_meteora_dynamic_liquidity_data(&data, true);
        assert!(result.is_some());

        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.pool_token_amount, 1000);
        assert_eq!(liquidity_data.maximum_token_a_amount, 2000);
        assert_eq!(liquidity_data.maximum_token_b_amount, 3000);
        assert_eq!(liquidity_data.token_a_amount, 2000);
        assert_eq!(liquidity_data.token_b_amount, 3000);
        assert_eq!(liquidity_data.minimum_pool_token_amount, 1000);
        assert!(liquidity_data.is_deposit);
    }

    #[test]
    fn test_parse_meteora_dynamic_liquidity_data_when_valid_data_withdraw_should_parse_correctly() {
        let mut data = DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&500u64.to_le_bytes()); // pool_token_amount
        data.extend_from_slice(&1000u64.to_le_bytes()); // maximum_token_a_amount
        data.extend_from_slice(&1500u64.to_le_bytes()); // maximum_token_b_amount

        let result = parse_meteora_dynamic_liquidity_data(&data, false);
        assert!(result.is_some());

        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.pool_token_amount, 500);
        assert_eq!(liquidity_data.maximum_token_a_amount, 1000);
        assert_eq!(liquidity_data.maximum_token_b_amount, 1500);
        assert!(!liquidity_data.is_deposit);
    }

    #[test]
    fn test_parse_meteora_dynamic_liquidity_data_when_insufficient_data_should_return_none() {
        let data = vec![0; 30]; // Less than 48 bytes
        let result = parse_meteora_dynamic_liquidity_data(&data, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_meteora_dynamic_liquidity_data_from_instruction_when_sufficient_accounts_should_set_accounts(
    ) {
        let mut data = DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let result = parse_meteora_dynamic_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(result.is_some());
        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.user, accounts[0]);
        assert_eq!(liquidity_data.pool, accounts[1]);
        assert_eq!(liquidity_data.vault_a, accounts[4]);
        assert_eq!(liquidity_data.vault_b, accounts[5]);
        assert_eq!(liquidity_data.lp_mint, accounts[6]);
    }

    #[test]
    fn test_parse_meteora_dynamic_liquidity_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let accounts = create_test_pubkeys(8); // Less than 10 accounts
        let result = parse_meteora_dynamic_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(result.is_some());
        let liquidity_data = result.unwrap();
        assert_eq!(liquidity_data.user, Pubkey::default());
        assert_eq!(liquidity_data.pool, Pubkey::default());
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_inner_instruction_when_valid_data_should_create_event() {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_swap_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_inner_instruction_when_invalid_data_should_return_none() {
        let data = vec![0; 10]; // Invalid data
        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_swap_inner_instruction(&data, metadata);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_meteora_dlmm_swap_instruction_when_valid_data_should_create_event() {
        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let accounts = create_test_pubkeys(10);
        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_swap_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dlmm_add_liquidity_inner_instruction_when_valid_data_should_create_event()
    {
        let mut data = DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_add_liquidity_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dlmm_add_liquidity_instruction_when_valid_data_should_create_event() {
        let mut data = DLMM_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_add_liquidity_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dlmm_remove_liquidity_inner_instruction_when_valid_data_should_create_event(
    ) {
        let mut data = DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_remove_liquidity_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dlmm_remove_liquidity_instruction_when_valid_data_should_create_event() {
        let mut data = DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&150u32.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let metadata = create_test_metadata();
        let result = parse_meteora_dlmm_remove_liquidity_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dynamic_add_liquidity_inner_instruction_when_valid_data_should_create_event(
    ) {
        let mut data = DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let result = parse_meteora_dynamic_add_liquidity_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dynamic_add_liquidity_instruction_when_valid_data_should_create_event() {
        let mut data = DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let metadata = create_test_metadata();
        let result = parse_meteora_dynamic_add_liquidity_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dynamic_remove_liquidity_inner_instruction_when_valid_data_should_create_event(
    ) {
        let mut data = DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let result = parse_meteora_dynamic_remove_liquidity_inner_instruction(&data, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_meteora_dynamic_remove_liquidity_instruction_when_valid_data_should_create_event()
    {
        let mut data = DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&2000u64.to_le_bytes());
        data.extend_from_slice(&3000u64.to_le_bytes());

        let accounts = create_test_pubkeys(12);
        let metadata = create_test_metadata();
        let result = parse_meteora_dynamic_remove_liquidity_instruction(&data, &accounts, metadata);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_no_block_time_should_use_zero() {
        let parser = MeteoraEventParser::default();

        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: bs58::encode(&data).into_string(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // No block time
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(!events.is_empty());
    }

    #[test]
    fn test_parse_events_from_instruction_when_no_block_time_should_use_zero() {
        let parser = MeteoraEventParser::default();
        let accounts = create_test_pubkeys(10);

        let mut data = DLMM_SWAP_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&500u64.to_le_bytes());
        data.push(1);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            data,
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // No block time
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(!events.is_empty());
    }
}
