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
    events::common::utils::{has_discriminator, parse_u32_le, parse_u64_le},
    events::core::{EventParser, GenericEventParseConfig},
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
        inner_instruction: &solana_transaction_status::UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        index.clone(),
                        program_received_time_ms,
                    );

                    if let Some(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&instruction.data, discriminator) {
                for config in configs {
                    let metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        index.clone(),
                        program_received_time_ms,
                    );

                    if let Some(event) =
                        (config.instruction_parser)(&instruction.data, accounts, metadata)
                    {
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
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_swap_data(data).map(|swap_data| {
        Box::new(MeteoraSwapEvent::new(
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
        )) as Box<dyn Event>
    })
}

fn parse_meteora_dlmm_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_swap_data_from_instruction(data, accounts).map(|swap_data| {
        Box::new(MeteoraSwapEvent::new(
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
        )) as Box<dyn Event>
    })
}

fn parse_meteora_dlmm_add_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_liquidity_data(data, true).map(|liquidity_data| {
        Box::new(MeteoraLiquidityEvent {
            id: metadata.id().to_string(),
            signature: metadata.signature.clone(),
            slot: metadata.slot,
            block_time: 0,    // Not available in SolanaEventMetadata
            block_time_ms: 0, // Not available in SolanaEventMetadata,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index.clone(),
            liquidity_data,
            transfer_data: Vec::new(),
            metadata: metadata.core,
        }) as Box<dyn Event>
    })
}

fn parse_meteora_dlmm_add_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_liquidity_data_from_instruction(data, accounts, true).map(|liquidity_data| {
        Box::new(MeteoraLiquidityEvent {
            id: metadata.id().to_string(),
            signature: metadata.signature.clone(),
            slot: metadata.slot,
            block_time: 0,    // Not available in SolanaEventMetadata
            block_time_ms: 0, // Not available in SolanaEventMetadata,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index.clone(),
            liquidity_data,
            transfer_data: Vec::new(),
            metadata: metadata.core,
        }) as Box<dyn Event>
    })
}

fn parse_meteora_dlmm_remove_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_liquidity_data(data, false).map(|liquidity_data| {
        Box::new(MeteoraLiquidityEvent {
            id: metadata.id().to_string(),
            signature: metadata.signature.clone(),
            slot: metadata.slot,
            block_time: 0,    // Not available in SolanaEventMetadata
            block_time_ms: 0, // Not available in SolanaEventMetadata,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index.clone(),
            liquidity_data,
            transfer_data: Vec::new(),
            metadata: metadata.core,
        }) as Box<dyn Event>
    })
}

fn parse_meteora_dlmm_remove_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dlmm_liquidity_data_from_instruction(data, accounts, false).map(
        |liquidity_data| {
            Box::new(MeteoraLiquidityEvent {
                id: metadata.id().to_string(),
                signature: metadata.signature.clone(),
                slot: metadata.slot,
                block_time: 0,    // Not available in SolanaEventMetadata
                block_time_ms: 0, // Not available in SolanaEventMetadata,
                program_received_time_ms: metadata.program_received_time_ms,
                program_handle_time_consuming_ms: 0,
                index: metadata.index.clone(),
                liquidity_data,
                transfer_data: Vec::new(),
                metadata: metadata.core,
            }) as Box<dyn Event>
        },
    )
}

// Parser functions for Dynamic AMM instructions

fn parse_meteora_dynamic_add_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dynamic_liquidity_data(data, true).map(|liquidity_data| {
        Box::new(MeteoraDynamicLiquidityEvent {
            id: metadata.id().to_string(),
            signature: metadata.signature.clone(),
            slot: metadata.slot,
            block_time: 0,    // Not available in SolanaEventMetadata
            block_time_ms: 0, // Not available in SolanaEventMetadata,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index.clone(),
            liquidity_data,
            transfer_data: Vec::new(),
            metadata: metadata.core,
        }) as Box<dyn Event>
    })
}

fn parse_meteora_dynamic_add_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dynamic_liquidity_data_from_instruction(data, accounts, true).map(
        |liquidity_data| {
            Box::new(MeteoraDynamicLiquidityEvent {
                id: metadata.id().to_string(),
                signature: metadata.signature.clone(),
                slot: metadata.slot,
                block_time: 0,    // Not available in SolanaEventMetadata
                block_time_ms: 0, // Not available in SolanaEventMetadata
                program_received_time_ms: metadata.program_received_time_ms,
                program_handle_time_consuming_ms: 0,
                index: metadata.index.clone(),
                liquidity_data,
                transfer_data: Vec::new(),
                metadata: metadata.core,
            }) as Box<dyn Event>
        },
    )
}

fn parse_meteora_dynamic_remove_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dynamic_liquidity_data(data, false).map(|liquidity_data| {
        Box::new(MeteoraDynamicLiquidityEvent {
            id: metadata.id().to_string(),
            signature: metadata.signature.clone(),
            slot: metadata.slot,
            block_time: 0,    // Not available in SolanaEventMetadata
            block_time_ms: 0, // Not available in SolanaEventMetadata,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index.clone(),
            liquidity_data,
            transfer_data: Vec::new(),
            metadata: metadata.core,
        }) as Box<dyn Event>
    })
}

fn parse_meteora_dynamic_remove_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_meteora_dynamic_liquidity_data_from_instruction(data, accounts, false).map(
        |liquidity_data| {
            Box::new(MeteoraDynamicLiquidityEvent {
                id: metadata.id().to_string(),
                signature: metadata.signature.clone(),
                slot: metadata.slot,
                block_time: 0,    // Not available in SolanaEventMetadata
                block_time_ms: 0, // Not available in SolanaEventMetadata
                program_received_time_ms: metadata.program_received_time_ms,
                program_handle_time_consuming_ms: 0,
                index: metadata.index.clone(),
                liquidity_data,
                transfer_data: Vec::new(),
                metadata: metadata.core,
            }) as Box<dyn Event>
        },
    )
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
