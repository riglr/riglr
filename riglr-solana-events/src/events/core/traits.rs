use std::fmt::Debug;
use crate::types::{EventMetadata, EventType, ProtocolType};
use riglr_events_core::Event;

// UnifiedEvent trait has been removed. Use riglr_events_core::Event instead.

/// Event Parser trait - defines the core methods for event parsing
/// This now returns Event trait objects instead of UnifiedEvent
#[async_trait::async_trait]
pub trait EventParser: Send + Sync {
    /// Get inner instruction parsing configurations
    fn inner_instruction_configs(&self) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>;
    
    /// Get instruction parsing configurations
    fn instruction_configs(&self) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>;
    
    /// Parse event data from inner instruction
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &solana_transaction_status::UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>>;

    /// Parse event data from instruction
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[solana_sdk::pubkey::Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn Event>>;

    /// Check if this program ID should be handled
    fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool;

    /// Get supported program ID list
    fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey>;
}

/// Generic event parser configuration
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    pub program_id: solana_sdk::pubkey::Pubkey,
    pub protocol_type: ProtocolType,
    pub inner_instruction_discriminator: &'static str,
    pub instruction_discriminator: &'static [u8],
    pub event_type: EventType,
    pub inner_instruction_parser: InnerInstructionEventParser,
    pub instruction_parser: InstructionEventParser,
}

/// Inner instruction event parser
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> Option<Box<dyn Event>>;

/// Instruction event parser
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[solana_sdk::pubkey::Pubkey], metadata: EventMetadata) -> Option<Box<dyn Event>>;

/// Generic event parser base class
pub struct GenericEventParser {
    pub program_ids: Vec<solana_sdk::pubkey::Pubkey>,
    pub inner_instruction_configs: std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>,
    pub instruction_configs: std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl GenericEventParser {
    /// Create new generic event parser
    pub fn new(program_ids: Vec<solana_sdk::pubkey::Pubkey>, configs: Vec<GenericEventParseConfig>) -> Self {
        use std::collections::HashMap;
        
        let mut inner_instruction_configs = HashMap::with_capacity(configs.len());
        let mut instruction_configs = HashMap::with_capacity(configs.len());

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

        Self { program_ids, inner_instruction_configs, instruction_configs }
    }
}

#[async_trait::async_trait]
impl EventParser for GenericEventParser {
    fn inner_instruction_configs(&self) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(&self) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
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
        accounts: &[solana_sdk::pubkey::Pubkey],
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

    fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.program_ids.clone()
    }
}