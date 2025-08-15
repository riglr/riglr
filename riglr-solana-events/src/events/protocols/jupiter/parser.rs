use std::collections::HashMap;
use riglr_events_core::Event;
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;
use crate::{
    events::core::{EventParser, GenericEventParseConfig},
    types::{EventMetadata, EventType, ProtocolType},
};
use super::{
    events::JupiterSwapEvent,
    types::{
        jupiter_v6_program_id, JupiterSwapData, RoutePlan,
        SharedAccountsRouteData, SharedAccountsExactOutRouteData,
        ROUTE_DISCRIMINATOR, EXACT_OUT_ROUTE_DISCRIMINATOR,
    },
};

/// Jupiter event parser
pub struct JupiterEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl JupiterEventParser {
    pub fn new() -> Self {
        let program_ids = vec![jupiter_v6_program_id()];
        
        let configs = vec![
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_swap_inner_instruction,
                instruction_parser: parse_jupiter_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: jupiter_v6_program_id(),
                protocol_type: ProtocolType::Other("Jupiter".to_string()),
                inner_instruction_discriminator: "exactOutRoute",
                instruction_discriminator: &EXACT_OUT_ROUTE_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_jupiter_exact_out_inner_instruction,
                instruction_parser: parse_jupiter_exact_out_instruction,
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
impl EventParser for JupiterEventParser {
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
        // since the program field isn't available in all Solana SDK versions
        if let Ok(data) = bs58::decode(&inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
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
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();
        
        if let Some(configs) = self.instruction_configs.get(&instruction.data) {
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
        
        events
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }

}

impl Default for JupiterEventParser {
    fn default() -> Self {
        Self::new()
    }
}

// Parser functions for different Jupiter instruction types

fn parse_jupiter_swap_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_jupiter_swap_with_borsh(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn Event>
    })
}

fn parse_jupiter_swap_instruction(
    data: &[u8],
    _accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_jupiter_swap_with_borsh(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn Event>
    })
}

fn parse_jupiter_exact_out_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_jupiter_exact_out_with_borsh(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn Event>
    })
}

fn parse_jupiter_exact_out_instruction(
    data: &[u8],
    _accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn Event>> {
    parse_jupiter_exact_out_with_borsh(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
            metadata.id,
            metadata.signature,
            metadata.slot,
            metadata.block_time,
            metadata.block_time_ms,
            metadata.program_received_time_ms,
            metadata.index,
            swap_data,
        )) as Box<dyn Event>
    })
}

// Data parsing helpers

/// Parse Jupiter swap data using borsh deserialization
fn parse_jupiter_swap_with_borsh(data: &[u8]) -> Option<JupiterSwapData> {
    // Skip discriminator (8 bytes)
    if data.len() < 8 {
        return None;
    }
    
    let instruction_data = &data[8..];
    
    // Try to deserialize as SharedAccountsRouteData
    if let Ok(route_data) = SharedAccountsRouteData::try_from_slice(instruction_data) {
        // Extract first and last swap info for simplified event data
        if let Some(first_step) = route_data.route_plan.first() {
            if let Some(last_step) = route_data.route_plan.last() {
                return Some(JupiterSwapData {
                    user: Pubkey::default(), // Will be set from accounts
                    input_mint: first_step.swap.source_token,
                    output_mint: last_step.swap.destination_token,
                    input_amount: route_data.in_amount,
                    output_amount: route_data.quoted_out_amount,
                    price_impact_pct: None,
                    platform_fee_bps: Some(route_data.platform_fee_bps as u32),
                    route_plan: route_data.route_plan.into_iter().map(|step| RoutePlan {
                        input_mint: step.swap.source_token,
                        output_mint: step.swap.destination_token,
                        amount_in: 0, // Not available in this format
                        amount_out: 0, // Not available in this format
                        dex_label: format!("DEX {}", step.percent),
                    }).collect(),
                });
            }
        }
    }
    
    None
}

/// Parse Jupiter exact out swap data using borsh deserialization
fn parse_jupiter_exact_out_with_borsh(data: &[u8]) -> Option<JupiterSwapData> {
    // Skip discriminator (8 bytes)
    if data.len() < 8 {
        return None;
    }
    
    let instruction_data = &data[8..];
    
    // Try to deserialize as SharedAccountsExactOutRouteData
    if let Ok(route_data) = SharedAccountsExactOutRouteData::try_from_slice(instruction_data) {
        // Extract first and last swap info for simplified event data
        if let Some(first_step) = route_data.route_plan.first() {
            if let Some(last_step) = route_data.route_plan.last() {
                return Some(JupiterSwapData {
                    user: Pubkey::default(), // Will be set from accounts
                    input_mint: first_step.swap.source_token,
                    output_mint: last_step.swap.destination_token,
                    input_amount: route_data.quoted_in_amount,
                    output_amount: route_data.out_amount,
                    price_impact_pct: None,
                    platform_fee_bps: Some(route_data.platform_fee_bps as u32),
                    route_plan: route_data.route_plan.into_iter().map(|step| RoutePlan {
                        input_mint: step.swap.source_token,
                        output_mint: step.swap.destination_token,
                        amount_in: 0, // Not available in this format
                        amount_out: 0, // Not available in this format
                        dex_label: format!("DEX {}", step.percent),
                    }).collect(),
                });
            }
        }
    }
    
    None
}


