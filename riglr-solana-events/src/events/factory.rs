use crate::events::core::EventParser;
use crate::types::ProtocolType;
use riglr_events_core::Event;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters for parsing events from instructions, reducing function parameter count
#[derive(Debug)]
pub struct InstructionParseParams<'a> {
    /// Compiled instruction data
    pub instruction: &'a solana_sdk::instruction::CompiledInstruction,
    /// Account keys from the transaction
    pub accounts: &'a [solana_sdk::pubkey::Pubkey],
    /// Transaction signature
    pub signature: &'a str,
    /// Solana slot number
    pub slot: u64,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Index string for event identification
    pub index: String,
}

/// Parameters for parsing events from inner instructions, reducing function parameter count
#[derive(Debug)]
pub struct InnerInstructionParseParams<'a> {
    /// Inner instruction data from transaction metadata
    pub inner_instruction: &'a solana_transaction_status::UiCompiledInstruction,
    /// Transaction signature
    pub signature: &'a str,
    /// Solana slot number
    pub slot: u64,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Index string for event identification
    pub index: String,
}

/// Protocol enum for supported protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    /// Orca Whirlpool concentrated liquidity protocol
    OrcaWhirlpool,
    /// Meteora Dynamic Liquidity Market Maker protocol
    MeteoraDlmm,
    /// MarginFi lending and borrowing protocol
    MarginFi,
    /// Jupiter swap aggregator protocol
    Jupiter,
    /// Raydium Automated Market Maker V4 protocol
    RaydiumAmmV4,
    /// Raydium Concentrated Liquidity Market Maker protocol
    RaydiumClmm,
    /// Raydium Constant Product Market Maker protocol
    RaydiumCpmm,
    /// PumpFun meme token creation protocol
    PumpFun,
    /// PumpSwap trading protocol
    PumpSwap,
    /// Bonk token protocol
    Bonk,
    /// Custom protocol with arbitrary name
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

/// EventParserRegistry - the new async event parser registry
#[derive(Default)]
pub struct EventParserRegistry {
    /// Map of protocols to their respective event parsers
    parsers: HashMap<Protocol, Arc<dyn EventParser>>,
    /// Map of program IDs to their respective event parsers for fast lookup
    program_id_to_parser: HashMap<solana_sdk::pubkey::Pubkey, Arc<dyn EventParser>>,
}

impl EventParserRegistry {
    /// Create a new event parser registry
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
    pub fn get_parser_for_program(
        &self,
        program_id: &solana_sdk::pubkey::Pubkey,
    ) -> Option<&Arc<dyn EventParser>> {
        self.program_id_to_parser.get(program_id)
    }

    /// Parse events from inner instruction using the appropriate parser
    pub fn parse_events_from_inner_instruction(
        &self,
        params: InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        // Try to identify the program and use the appropriate parser
        for parser in self.parsers.values() {
            let events = parser.parse_events_from_inner_instruction(
                params.inner_instruction,
                params.signature,
                params.slot,
                params.block_time,
                params.program_received_time_ms,
                params.index.clone(),
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
        params: InstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        // Get the program ID from the instruction
        if let Some(program_id) = params
            .accounts
            .get(params.instruction.program_id_index as usize)
        {
            if let Some(parser) = self.get_parser_for_program(program_id) {
                return parser.parse_events_from_instruction(
                    params.instruction,
                    params.accounts,
                    params.signature,
                    params.slot,
                    params.block_time,
                    params.program_received_time_ms,
                    params.index,
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

    /// Create a registry with all available parsers
    pub fn with_all_parsers() -> Self {
        // Add parsers for all supported protocols
        // Note: Specific protocol parsers will be added here once they're updated

        Self::new()
    }
}
