use std::any::Any;
use std::fmt::Debug;
use crate::types::{EventMetadata, EventType, ProtocolType, TransferData, SwapData};

/// Unified Event Interface - All protocol events must implement this trait
pub trait UnifiedEvent: Debug + Send + Sync {
    /// Get event ID
    fn id(&self) -> &str;

    /// Get event type
    fn event_type(&self) -> EventType;

    /// Get transaction signature
    fn signature(&self) -> &str;

    /// Get slot number
    fn slot(&self) -> u64;

    /// Get program received timestamp (milliseconds)
    fn program_received_time_ms(&self) -> i64;

    /// Processing time consumption (milliseconds)
    fn program_handle_time_consuming_ms(&self) -> i64;

    /// Set processing time consumption (milliseconds)
    fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64);

    /// Convert event to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Convert event to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone the event
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent>;

    /// Merge events (optional implementation)
    fn merge(&mut self, _other: Box<dyn UnifiedEvent>) {
        // Default implementation: no merging operation
    }

    /// Set transfer data
    fn set_transfer_data(
        &mut self,
        transfer_data: Vec<TransferData>,
        swap_data: Option<SwapData>,
    );

    /// Get index
    fn index(&self) -> String;

    /// Get protocol type
    fn protocol_type(&self) -> ProtocolType;

    /// Get timestamp
    fn timestamp(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_millis(self.program_received_time_ms() as u64)
    }

    /// Get transaction hash
    fn transaction_hash(&self) -> Option<String> {
        Some(self.signature().to_string())
    }

    /// Get block number
    fn block_number(&self) -> Option<u64> {
        Some(self.slot())
    }

    // Core event methods only - streaming logic moved to riglr-streams
}

/// Implement Clone for Box<dyn UnifiedEvent>
impl Clone for Box<dyn UnifiedEvent> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Event Parser trait - defines the core methods for event parsing
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
    ) -> Vec<Box<dyn UnifiedEvent>>;

    /// Parse event data from instruction
    fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[solana_sdk::pubkey::Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>>;

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
    fn(data: &[u8], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

/// Instruction event parser
pub type InstructionEventParser =
    fn(data: &[u8], accounts: &[solana_sdk::pubkey::Pubkey], metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

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
        accounts: &[solana_sdk::pubkey::Pubkey],
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

    fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.program_ids.clone()
    }
}