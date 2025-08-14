use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use crate::{
    events::core::{EventParser, GenericEventParseConfig, UnifiedEvent},
    types::{EventMetadata, EventType, ProtocolType},
    events::common::utils::{has_discriminator, parse_u64_le, parse_u128_le, parse_u32_le},
};
use super::{
    events::{OrcaSwapEvent, OrcaPositionEvent, OrcaLiquidityEvent},
    types::{
        orca_whirlpool_program_id, OrcaSwapData, OrcaPositionData, OrcaLiquidityData,
        SWAP_DISCRIMINATOR, OPEN_POSITION_DISCRIMINATOR, CLOSE_POSITION_DISCRIMINATOR,
        INCREASE_LIQUIDITY_DISCRIMINATOR, DECREASE_LIQUIDITY_DISCRIMINATOR,
        PositionRewardInfo,
    },
};

/// Orca Whirlpool event parser
pub struct OrcaEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl OrcaEventParser {
    pub fn new() -> Self {
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

#[async_trait::async_trait]
impl EventParser for OrcaEventParser {
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
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut events = Vec::new();
        
        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&inner_instruction.data).into_vec() {
            for (_, configs) in &self.inner_instruction_configs {
                for config in configs {
                    let metadata = EventMetadata::new(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        block_time.unwrap_or(0) * 1000,
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
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut events = Vec::new();
        
        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&instruction.data, discriminator) {
                for config in configs {
                    let metadata = EventMetadata::new(
                        format!("{}_{}", signature, index),
                        signature.to_string(),
                        slot,
                        block_time.unwrap_or(0),
                        block_time.unwrap_or(0) * 1000,
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                        config.program_id,
                        index.clone(),
                        program_received_time_ms,
                    );
                    
                    if let Some(event) = (config.instruction_parser)(&instruction.data, accounts, metadata) {
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
        Self::new()
    }
}

// Parser functions for different Orca instruction types

fn parse_orca_swap_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_swap_data(data).map(|swap_data| {
        Box::new(OrcaSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_swap_data_from_instruction(data, accounts).map(|swap_data| {
        Box::new(OrcaSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_open_position_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_position_data(data).map(|position_data| {
        Box::new(OrcaPositionEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            position_data,
            is_open: true,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_open_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_position_data_from_instruction(data, accounts).map(|position_data| {
        Box::new(OrcaPositionEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            position_data,
            is_open: true,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_close_position_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_position_data(data).map(|position_data| {
        Box::new(OrcaPositionEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            position_data,
            is_open: false,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_close_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_position_data_from_instruction(data, accounts).map(|position_data| {
        Box::new(OrcaPositionEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            position_data,
            is_open: false,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_increase_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_liquidity_data(data, true).map(|liquidity_data| {
        Box::new(OrcaLiquidityEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidity_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_increase_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_liquidity_data_from_instruction(data, accounts, true).map(|liquidity_data| {
        Box::new(OrcaLiquidityEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidity_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_decrease_liquidity_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_liquidity_data(data, false).map(|liquidity_data| {
        Box::new(OrcaLiquidityEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidity_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

fn parse_orca_decrease_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_orca_liquidity_data_from_instruction(data, accounts, false).map(|liquidity_data| {
        Box::new(OrcaLiquidityEvent {
            id: metadata.id,
            signature: metadata.signature,
            slot: metadata.slot,
            block_time: metadata.block_time,
            block_time_ms: metadata.block_time_ms,
            program_received_time_ms: metadata.program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            index: metadata.index,
            liquidity_data,
            transfer_data: Vec::new(),
        }) as Box<dyn UnifiedEvent>
    })
}

// Data parsing helpers

fn parse_orca_swap_data(data: &[u8]) -> Option<OrcaSwapData> {
    if data.len() < 64 {
        return None;
    }

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
        whirlpool: Pubkey::default(), // Would need to extract from accounts
        user: Pubkey::default(), // Would need to extract from accounts
        token_mint_a: Pubkey::default(), // Would need to extract from accounts
        token_mint_b: Pubkey::default(), // Would need to extract from accounts
        token_vault_a: Pubkey::default(), // Would need to extract from accounts
        token_vault_b: Pubkey::default(), // Would need to extract from accounts
        amount,
        amount_specified_is_input,
        a_to_b,
        sqrt_price_limit,
        amount_in: if amount_specified_is_input { amount } else { other_amount_threshold },
        amount_out: if amount_specified_is_input { other_amount_threshold } else { amount },
        fee_amount: 0, // Would need to calculate from pool state
        tick_current_index: 0, // Would need to extract from pool state
        sqrt_price: 0, // Would need to extract from pool state
        liquidity: 0, // Would need to extract from pool state
    })
}

fn parse_orca_swap_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<OrcaSwapData> {
    let mut swap_data = parse_orca_swap_data(data)?;
    
    // Extract accounts (typical Orca swap instruction layout)
    if accounts.len() >= 11 {
        swap_data.whirlpool = accounts[1];
        swap_data.user = accounts[0];
        swap_data.token_vault_a = accounts[3];
        swap_data.token_vault_b = accounts[4];
        // Note: Would need more sophisticated parsing for all fields
    }
    
    Some(swap_data)
}

fn parse_orca_position_data(data: &[u8]) -> Option<OrcaPositionData> {
    if data.len() < 32 {
        return None;
    }

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

fn parse_orca_position_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<OrcaPositionData> {
    let mut position_data = parse_orca_position_data(data)?;
    
    // Extract accounts from instruction
    if accounts.len() >= 7 {
        position_data.whirlpool = accounts[1];
        position_data.position_authority = accounts[0];
        position_data.position = accounts[2];
        position_data.position_mint = accounts[3];
        position_data.position_token_account = accounts[4];
    }
    
    Some(position_data)
}

fn parse_orca_liquidity_data(data: &[u8], is_increase: bool) -> Option<OrcaLiquidityData> {
    if data.len() < 40 {
        return None;
    }

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

fn parse_orca_liquidity_data_from_instruction(data: &[u8], accounts: &[Pubkey], is_increase: bool) -> Option<OrcaLiquidityData> {
    let mut liquidity_data = parse_orca_liquidity_data(data, is_increase)?;
    
    // Extract accounts from instruction
    if accounts.len() >= 12 {
        liquidity_data.whirlpool = accounts[1];
        liquidity_data.position_authority = accounts[0];
        liquidity_data.position = accounts[2];
        liquidity_data.token_vault_a = accounts[5];
        liquidity_data.token_vault_b = accounts[6];
    }
    
    Some(liquidity_data)
}

impl Default for PositionRewardInfo {
    fn default() -> Self {
        Self {
            growth_inside_checkpoint: 0,
            amount_owed: 0,
        }
    }
}