use std::collections::HashMap;
use std::sync::Arc;
use crate::events::core::{EventParser, UnifiedEvent};
use crate::types::ProtocolType;

/// Protocol enum for supported protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    OrcaWhirlpool,
    MeteoraDlmm,
    MarginFi,
    Jupiter,
    RaydiumAmmV4,
    RaydiumClmm,
    RaydiumCpmm,
    PumpFun,
    PumpSwap,
    Bonk,
    Custom(String),
}

impl From<ProtocolType> for Protocol {
    fn from(protocol_type: ProtocolType) -> Self {
        match protocol_type {
            ProtocolType::OrcaWhirlpool => Protocol::OrcaWhirlpool,
            ProtocolType::MeteoraDlmm => Protocol::MeteoraDlmm,
            ProtocolType::MarginFi => Protocol::MarginFi,
            ProtocolType::Bonk => Protocol::Bonk,
            ProtocolType::PumpSwap => Protocol::PumpSwap,
            ProtocolType::RaydiumAmm => Protocol::RaydiumAmmV4,
            ProtocolType::RaydiumAmmV4 => Protocol::RaydiumAmmV4,
            ProtocolType::RaydiumClmm => Protocol::RaydiumClmm,
            ProtocolType::RaydiumCpmm => Protocol::RaydiumCpmm,
            ProtocolType::Jupiter => Protocol::Jupiter,
            ProtocolType::Other(name) => match name.as_str() {
                "Jupiter" => Protocol::Jupiter,
                "RaydiumAmmV4" => Protocol::RaydiumAmmV4,
                "RaydiumClmm" => Protocol::RaydiumClmm,
                "RaydiumCpmm" => Protocol::RaydiumCpmm,
                "PumpFun" => Protocol::PumpFun,
                "Bonk" => Protocol::Bonk,
                _ => Protocol::Custom(name),
            },
        }
    }
}

impl From<Protocol> for ProtocolType {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::OrcaWhirlpool => ProtocolType::OrcaWhirlpool,
            Protocol::MeteoraDlmm => ProtocolType::MeteoraDlmm,
            Protocol::MarginFi => ProtocolType::MarginFi,
            Protocol::Jupiter => ProtocolType::Jupiter,
            Protocol::RaydiumAmmV4 => ProtocolType::RaydiumAmmV4,
            Protocol::RaydiumClmm => ProtocolType::RaydiumClmm,
            Protocol::RaydiumCpmm => ProtocolType::RaydiumCpmm,
            Protocol::PumpFun => ProtocolType::Other("PumpFun".to_string()),
            Protocol::PumpSwap => ProtocolType::PumpSwap,
            Protocol::Bonk => ProtocolType::Bonk,
            Protocol::Custom(name) => ProtocolType::Other(name),
        }
    }
}

/// Multi-event parser that combines multiple protocol parsers
pub struct MultiEventParser {
    parsers: HashMap<Protocol, Arc<dyn EventParser>>,
    program_id_to_parser: HashMap<solana_sdk::pubkey::Pubkey, Arc<dyn EventParser>>,
}

impl MultiEventParser {
    /// Create a new multi-event parser
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            program_id_to_parser: HashMap::new(),
        }
    }

    /// Add a parser for a specific protocol
    pub fn add_parser(&mut self, protocol: Protocol, parser: Arc<dyn EventParser>) {
        // Map all supported program IDs to this parser
        for program_id in parser.supported_program_ids() {
            self.program_id_to_parser.insert(program_id, parser.clone());
        }
        self.parsers.insert(protocol, parser);
    }

    /// Get parser for a specific protocol
    pub fn get_parser(&self, protocol: &Protocol) -> Option<&Arc<dyn EventParser>> {
        self.parsers.get(protocol)
    }

    /// Get parser for a specific program ID
    pub fn get_parser_for_program(&self, program_id: &solana_sdk::pubkey::Pubkey) -> Option<&Arc<dyn EventParser>> {
        self.program_id_to_parser.get(program_id)
    }

    /// Parse events from inner instruction using the appropriate parser
    pub fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &solana_transaction_status::UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        // Try to identify the program and use the appropriate parser
        for parser in self.parsers.values() {
            let events = parser.parse_events_from_inner_instruction(
                inner_instruction,
                signature,
                slot,
                block_time,
                program_received_time_ms,
                index.clone(),
            );
            if !events.is_empty() {
                return events;
            }
        }
        vec![]
    }

    /// Parse events from instruction using the appropriate parser
    pub fn parse_events_from_instruction(
        &self,
        instruction: &solana_sdk::instruction::CompiledInstruction,
        accounts: &[solana_sdk::pubkey::Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        // Get the program ID from the instruction
        if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
            if let Some(parser) = self.get_parser_for_program(program_id) {
                return parser.parse_events_from_instruction(
                    instruction,
                    accounts,
                    signature,
                    slot,
                    block_time,
                    program_received_time_ms,
                    index,
                );
            }
        }
        vec![]
    }

    /// Get all supported program IDs
    pub fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.program_id_to_parser.keys().cloned().collect()
    }

    /// Check if a program ID is supported
    pub fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.program_id_to_parser.contains_key(program_id)
    }
}

impl Default for MultiEventParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Event parser factory for creating and managing parsers
pub struct EventParserFactory {
    multi_parser: MultiEventParser,
}

impl EventParserFactory {
    /// Create a new event parser factory
    pub fn new() -> Self {
        Self {
            multi_parser: MultiEventParser::new(),
        }
    }

    /// Create a factory with all available parsers
    pub fn with_all_parsers() -> Self {
        let factory = Self::new();
        
        // Add parsers for all supported protocols
        // Note: Specific protocol parsers will be implemented in their respective modules
        
        factory
    }

    /// Add a parser for a specific protocol
    pub fn add_parser(&mut self, protocol: Protocol, parser: Arc<dyn EventParser>) -> &mut Self {
        self.multi_parser.add_parser(protocol, parser);
        self
    }

    /// Build the multi-event parser
    pub fn build(self) -> MultiEventParser {
        self.multi_parser
    }

    /// Get a reference to the multi-parser
    pub fn parser(&self) -> &MultiEventParser {
        &self.multi_parser
    }
}

impl Default for EventParserFactory {
    fn default() -> Self {
        Self::new()
    }
}