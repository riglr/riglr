use std::collections::HashMap;
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;
use crate::{
    events::core::{EventParser, GenericEventParseConfig, UnifiedEvent},
    types::{EventMetadata, EventType, ProtocolType},
    events::common::utils::{has_discriminator, parse_u64_le, parse_pubkey_from_bytes},
};
use super::{
    events::JupiterSwapEvent,
    types::{
        jupiter_v6_program_id, JupiterSwapData, RoutePlan,
        SharedAccountsRouteData, SharedAccountsExactOutRouteData,
        ROUTE_DISCRIMINATOR, EXACT_OUT_ROUTE_DISCRIMINATOR,
        LEGACY_ROUTE_DISCRIMINATOR, LEGACY_EXACT_OUT_DISCRIMINATOR,
        SWAP_DISCRIMINATOR, ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR,
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
    ) -> Vec<Box<dyn UnifiedEvent>> {
        let mut events = Vec::new();
        
        // For inner instructions, we'll use the data to identify the instruction type
        // since the program field isn't available in all Solana SDK versions
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
) -> Option<Box<dyn UnifiedEvent>> {
    parse_jupiter_swap_data(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
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

fn parse_jupiter_swap_instruction(
    data: &[u8],
    _accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_jupiter_swap_data(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
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

fn parse_jupiter_exact_out_inner_instruction(
    data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_jupiter_exact_out_data(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
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

fn parse_jupiter_exact_out_instruction(
    data: &[u8],
    _accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    parse_jupiter_exact_out_data(data).map(|swap_data| {
        Box::new(JupiterSwapEvent::new(
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

// Legacy manual parsing (kept for backward compatibility)
fn parse_jupiter_swap_data(data: &[u8]) -> Option<JupiterSwapData> {
    // First try borsh deserialization
    if let Some(swap_data) = parse_jupiter_swap_with_borsh(data) {
        return Some(swap_data);
    }
    
    // Fall back to manual parsing
    if data.len() < 72 { // Minimum required for discriminator + basic fields
        return None;
    }

    let mut offset = 8; // Skip discriminator

    // Parse route plan count
    let route_plan_count = data.get(offset)?;
    offset += 1;

    let mut route_plan = Vec::new();
    for _ in 0..*route_plan_count {
        if offset + 88 > data.len() { // Each route plan entry is ~88 bytes
            break;
        }

        let input_mint = parse_pubkey_from_bytes(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        let output_mint = parse_pubkey_from_bytes(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        let amount_in = parse_u64_le(&data[offset..offset + 8]).ok()?;
        offset += 8;
        
        let amount_out = parse_u64_le(&data[offset..offset + 8]).ok()?;
        offset += 8;

        // Skip dex info for now (simplified parsing)
        offset += 8;

        route_plan.push(RoutePlan {
            input_mint,
            output_mint,
            amount_in,
            amount_out,
            dex_label: "Unknown".to_string(),
        });
    }

    if route_plan.is_empty() {
        return None;
    }

    // Extract overall swap info from first and last route
    let first_route = &route_plan[0];
    let last_route = &route_plan[route_plan.len() - 1];

    Some(JupiterSwapData {
        user: Pubkey::default(), // Would need to extract from accounts
        input_mint: first_route.input_mint,
        output_mint: last_route.output_mint,
        input_amount: first_route.amount_in,
        output_amount: last_route.amount_out,
        price_impact_pct: None,
        platform_fee_bps: None,
        route_plan,
    })
}

fn parse_jupiter_exact_out_data(data: &[u8]) -> Option<JupiterSwapData> {
    // First try borsh deserialization
    if let Some(swap_data) = parse_jupiter_exact_out_with_borsh(data) {
        return Some(swap_data);
    }
    
    // Fall back to manual parsing
    if data.len() < 80 {
        return None;
    }

    let mut offset = 8; // Skip discriminator

    // Parse exact out specific fields
    let out_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;
    
    let quote_max_in_amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    // Parse route plan (simplified)
    let route_plan_count = data.get(offset)?;
    offset += 1;

    let mut route_plan = Vec::new();
    for _ in 0..*route_plan_count {
        if offset + 88 > data.len() {
            break;
        }

        let input_mint = parse_pubkey_from_bytes(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        let output_mint = parse_pubkey_from_bytes(&data[offset..offset + 32]).ok()?;
        offset += 32;
        
        offset += 16; // Skip amounts for now
        offset += 8; // Skip dex info

        route_plan.push(RoutePlan {
            input_mint,
            output_mint,
            amount_in: quote_max_in_amount,
            amount_out: out_amount,
            dex_label: "Unknown".to_string(),
        });
    }

    if route_plan.is_empty() {
        return None;
    }

    let first_route = &route_plan[0];
    let last_route = &route_plan[route_plan.len() - 1];

    Some(JupiterSwapData {
        user: Pubkey::default(),
        input_mint: first_route.input_mint,
        output_mint: last_route.output_mint,
        input_amount: quote_max_in_amount,
        output_amount: out_amount,
        price_impact_pct: None,
        platform_fee_bps: None,
        route_plan,
    })
}

