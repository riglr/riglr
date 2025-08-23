use crate::events::core::EventParser;
use crate::events::protocols::{
    bonk_parser::BonkEventParser, jupiter_parser::JupiterEventParser,
    marginfi_parser::MarginFiEventParser, meteora_parser::MeteoraEventParser,
    orca_parser::OrcaEventParser, pumpswap_parser::PumpSwapEventParser,
    raydium_clmm_parser::RaydiumClmmEventParser, raydium_cpmm_parser::RaydiumCpmmEventParser,
    raydium_v4_parser::RaydiumAmmV4EventParser,
};
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
            ProtocolType::Raydium => Protocol::RaydiumAmmV4, // Default general Raydium to AMM V4
            ProtocolType::Jupiter => Protocol::Jupiter,
            ProtocolType::Serum => Protocol::Custom("Serum".to_string()),
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
            let events = parser.parse_events_from_inner_instruction(&params);
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
                return parser.parse_events_from_instruction(&params);
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
        let mut registry = Self::new();

        // Add parsers for all supported protocols
        registry.add_parser(Protocol::Bonk, Arc::new(BonkEventParser::new()));
        registry.add_parser(Protocol::Jupiter, Arc::new(JupiterEventParser::new()));
        registry.add_parser(Protocol::MarginFi, Arc::new(MarginFiEventParser::default()));
        registry.add_parser(Protocol::MeteoraDlmm, Arc::new(MeteoraEventParser::new()));
        registry.add_parser(
            Protocol::OrcaWhirlpool,
            Arc::new(OrcaEventParser::default()),
        );
        registry.add_parser(Protocol::PumpSwap, Arc::new(PumpSwapEventParser::new()));
        registry.add_parser(
            Protocol::RaydiumAmmV4,
            Arc::new(RaydiumAmmV4EventParser::default()),
        );
        registry.add_parser(
            Protocol::RaydiumClmm,
            Arc::new(RaydiumClmmEventParser::new()),
        );
        registry.add_parser(
            Protocol::RaydiumCpmm,
            Arc::new(RaydiumCpmmEventParser::new()),
        );

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::Event;
    use solana_sdk::instruction::CompiledInstruction;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiCompiledInstruction;
    use std::collections::HashMap;

    // Mock EventParser for testing
    struct MockEventParser {
        program_ids: Vec<Pubkey>,
        returns_events: bool,
    }

    impl MockEventParser {
        fn new(program_ids: Vec<Pubkey>, returns_events: bool) -> Self {
            Self {
                program_ids,
                returns_events,
            }
        }
    }

    impl EventParser for MockEventParser {
        fn inner_instruction_configs(
            &self,
        ) -> std::collections::HashMap<
            &'static str,
            Vec<crate::events::core::GenericEventParseConfig>,
        > {
            std::collections::HashMap::new()
        }

        fn instruction_configs(
            &self,
        ) -> std::collections::HashMap<Vec<u8>, Vec<crate::events::core::GenericEventParseConfig>>
        {
            std::collections::HashMap::new()
        }

        fn should_handle(&self, program_id: &Pubkey) -> bool {
            self.program_ids.contains(program_id)
        }

        fn supported_program_ids(&self) -> Vec<Pubkey> {
            self.program_ids.clone()
        }

        fn parse_events_from_instruction(
            &self,
            _params: &InstructionParseParams,
        ) -> Vec<Box<dyn Event>> {
            if self.returns_events {
                vec![Box::new(MockEvent::default())]
            } else {
                vec![]
            }
        }

        fn parse_events_from_inner_instruction(
            &self,
            _params: &InnerInstructionParseParams,
        ) -> Vec<Box<dyn Event>> {
            if self.returns_events {
                vec![Box::new(MockEvent::default())]
            } else {
                vec![]
            }
        }
    }

    // Mock Event for testing
    #[derive(Debug)]
    struct MockEvent {
        metadata: riglr_events_core::types::EventMetadata,
    }

    impl Default for MockEvent {
        fn default() -> Self {
            Self {
                metadata: riglr_events_core::types::EventMetadata::new(
                    "mock_event".to_string(),
                    riglr_events_core::types::EventKind::Transaction,
                    "mock".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &riglr_events_core::types::EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &riglr_events_core::types::EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut riglr_events_core::types::EventMetadata {
            &mut self.metadata
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(MockEvent {
                metadata: self.metadata.clone(),
            })
        }
    }

    // Tests for InstructionParseParams
    #[test]
    fn test_instruction_parse_params_creation() {
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![Pubkey::new_unique()];
        let signature = "test_signature";
        let slot = 12345;
        let block_time = Some(1234567890);
        let program_received_time_ms = 1234567890123;
        let index = "0".to_string();

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index: index.clone(),
        };

        assert_eq!(params.signature, signature);
        assert_eq!(params.slot, slot);
        assert_eq!(params.block_time, block_time);
        assert_eq!(params.program_received_time_ms, program_received_time_ms);
        assert_eq!(params.index, index);
    }

    // Tests for InnerInstructionParseParams
    #[test]
    fn test_inner_instruction_parse_params_creation() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };
        let signature = "test_signature";
        let slot = 12345;
        let block_time = Some(1234567890);
        let program_received_time_ms = 1234567890123;
        let index = "0".to_string();

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index: index.clone(),
        };

        assert_eq!(params.signature, signature);
        assert_eq!(params.slot, slot);
        assert_eq!(params.block_time, block_time);
        assert_eq!(params.program_received_time_ms, program_received_time_ms);
        assert_eq!(params.index, index);
    }

    // Tests for Protocol enum
    #[test]
    fn test_protocol_variants() {
        let protocols = vec![
            Protocol::OrcaWhirlpool,
            Protocol::MeteoraDlmm,
            Protocol::MarginFi,
            Protocol::Jupiter,
            Protocol::RaydiumAmmV4,
            Protocol::RaydiumClmm,
            Protocol::RaydiumCpmm,
            Protocol::PumpFun,
            Protocol::PumpSwap,
            Protocol::Bonk,
            Protocol::Custom("TestProtocol".to_string()),
        ];

        for protocol in protocols {
            assert!(matches!(
                protocol,
                Protocol::OrcaWhirlpool
                    | Protocol::MeteoraDlmm
                    | Protocol::MarginFi
                    | Protocol::Jupiter
                    | Protocol::RaydiumAmmV4
                    | Protocol::RaydiumClmm
                    | Protocol::RaydiumCpmm
                    | Protocol::PumpFun
                    | Protocol::PumpSwap
                    | Protocol::Bonk
                    | Protocol::Custom(_)
            ));
        }
    }

    #[test]
    fn test_protocol_equality() {
        assert_eq!(Protocol::OrcaWhirlpool, Protocol::OrcaWhirlpool);
        assert_eq!(
            Protocol::Custom("test".to_string()),
            Protocol::Custom("test".to_string())
        );
        assert_ne!(Protocol::OrcaWhirlpool, Protocol::MeteoraDlmm);
        assert_ne!(
            Protocol::Custom("test1".to_string()),
            Protocol::Custom("test2".to_string())
        );
    }

    #[test]
    fn test_protocol_clone() {
        let protocol = Protocol::Custom("test".to_string());
        let cloned = protocol.clone();
        assert_eq!(protocol, cloned);
    }

    #[test]
    fn test_protocol_hash() {
        let mut map = HashMap::new();
        map.insert(Protocol::OrcaWhirlpool, "orca");
        map.insert(Protocol::Custom("test".to_string()), "custom");

        assert_eq!(map.get(&Protocol::OrcaWhirlpool), Some(&"orca"));
        assert_eq!(
            map.get(&Protocol::Custom("test".to_string())),
            Some(&"custom")
        );
    }

    // Tests for Protocol -> ProtocolType conversion
    #[test]
    fn test_protocol_from_protocol_type_orca_whirlpool() {
        let protocol = Protocol::from(ProtocolType::OrcaWhirlpool);
        assert_eq!(protocol, Protocol::OrcaWhirlpool);
    }

    #[test]
    fn test_protocol_from_protocol_type_meteora_dlmm() {
        let protocol = Protocol::from(ProtocolType::MeteoraDlmm);
        assert_eq!(protocol, Protocol::MeteoraDlmm);
    }

    #[test]
    fn test_protocol_from_protocol_type_margin_fi() {
        let protocol = Protocol::from(ProtocolType::MarginFi);
        assert_eq!(protocol, Protocol::MarginFi);
    }

    #[test]
    fn test_protocol_from_protocol_type_bonk() {
        let protocol = Protocol::from(ProtocolType::Bonk);
        assert_eq!(protocol, Protocol::Bonk);
    }

    #[test]
    fn test_protocol_from_protocol_type_pump_swap() {
        let protocol = Protocol::from(ProtocolType::PumpSwap);
        assert_eq!(protocol, Protocol::PumpSwap);
    }

    #[test]
    fn test_protocol_from_protocol_type_raydium_amm() {
        let protocol = Protocol::from(ProtocolType::RaydiumAmm);
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn test_protocol_from_protocol_type_raydium_amm_v4() {
        let protocol = Protocol::from(ProtocolType::RaydiumAmmV4);
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn test_protocol_from_protocol_type_raydium_clmm() {
        let protocol = Protocol::from(ProtocolType::RaydiumClmm);
        assert_eq!(protocol, Protocol::RaydiumClmm);
    }

    #[test]
    fn test_protocol_from_protocol_type_raydium_cpmm() {
        let protocol = Protocol::from(ProtocolType::RaydiumCpmm);
        assert_eq!(protocol, Protocol::RaydiumCpmm);
    }

    #[test]
    fn test_protocol_from_protocol_type_jupiter() {
        let protocol = Protocol::from(ProtocolType::Jupiter);
        assert_eq!(protocol, Protocol::Jupiter);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_jupiter() {
        let protocol = Protocol::from(ProtocolType::Other("Jupiter".to_string()));
        assert_eq!(protocol, Protocol::Jupiter);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_raydium_amm_v4() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumAmmV4".to_string()));
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_raydium_clmm() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumClmm".to_string()));
        assert_eq!(protocol, Protocol::RaydiumClmm);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_raydium_cpmm() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumCpmm".to_string()));
        assert_eq!(protocol, Protocol::RaydiumCpmm);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_pump_fun() {
        let protocol = Protocol::from(ProtocolType::Other("PumpFun".to_string()));
        assert_eq!(protocol, Protocol::PumpFun);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_bonk() {
        let protocol = Protocol::from(ProtocolType::Other("Bonk".to_string()));
        assert_eq!(protocol, Protocol::Bonk);
    }

    #[test]
    fn test_protocol_from_protocol_type_other_custom() {
        let protocol = Protocol::from(ProtocolType::Other("CustomProtocol".to_string()));
        assert_eq!(protocol, Protocol::Custom("CustomProtocol".to_string()));
    }

    // Tests for ProtocolType -> Protocol conversion
    #[test]
    fn test_protocol_type_from_protocol_orca_whirlpool() {
        let protocol_type = ProtocolType::from(Protocol::OrcaWhirlpool);
        assert_eq!(protocol_type, ProtocolType::OrcaWhirlpool);
    }

    #[test]
    fn test_protocol_type_from_protocol_meteora_dlmm() {
        let protocol_type = ProtocolType::from(Protocol::MeteoraDlmm);
        assert_eq!(protocol_type, ProtocolType::MeteoraDlmm);
    }

    #[test]
    fn test_protocol_type_from_protocol_margin_fi() {
        let protocol_type = ProtocolType::from(Protocol::MarginFi);
        assert_eq!(protocol_type, ProtocolType::MarginFi);
    }

    #[test]
    fn test_protocol_type_from_protocol_jupiter() {
        let protocol_type = ProtocolType::from(Protocol::Jupiter);
        assert_eq!(protocol_type, ProtocolType::Jupiter);
    }

    #[test]
    fn test_protocol_type_from_protocol_raydium_amm_v4() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumAmmV4);
        assert_eq!(protocol_type, ProtocolType::RaydiumAmmV4);
    }

    #[test]
    fn test_protocol_type_from_protocol_raydium_clmm() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumClmm);
        assert_eq!(protocol_type, ProtocolType::RaydiumClmm);
    }

    #[test]
    fn test_protocol_type_from_protocol_raydium_cpmm() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumCpmm);
        assert_eq!(protocol_type, ProtocolType::RaydiumCpmm);
    }

    #[test]
    fn test_protocol_type_from_protocol_pump_fun() {
        let protocol_type = ProtocolType::from(Protocol::PumpFun);
        assert_eq!(protocol_type, ProtocolType::Other("PumpFun".to_string()));
    }

    #[test]
    fn test_protocol_type_from_protocol_pump_swap() {
        let protocol_type = ProtocolType::from(Protocol::PumpSwap);
        assert_eq!(protocol_type, ProtocolType::PumpSwap);
    }

    #[test]
    fn test_protocol_type_from_protocol_bonk() {
        let protocol_type = ProtocolType::from(Protocol::Bonk);
        assert_eq!(protocol_type, ProtocolType::Bonk);
    }

    #[test]
    fn test_protocol_type_from_protocol_custom() {
        let protocol_type = ProtocolType::from(Protocol::Custom("CustomProtocol".to_string()));
        assert_eq!(
            protocol_type,
            ProtocolType::Other("CustomProtocol".to_string())
        );
    }

    // Tests for EventParserRegistry
    #[test]
    fn test_event_parser_registry_new() {
        let registry = EventParserRegistry::default();
        assert!(registry.parsers.is_empty());
        assert!(registry.program_id_to_parser.is_empty());
    }

    #[test]
    fn test_event_parser_registry_default() {
        let registry = EventParserRegistry::default();
        assert!(registry.parsers.is_empty());
        assert!(registry.program_id_to_parser.is_empty());
    }

    #[test]
    fn test_event_parser_registry_add_parser() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        assert_eq!(registry.parsers.len(), 1);
        assert_eq!(registry.program_id_to_parser.len(), 1);
        assert!(registry.parsers.contains_key(&Protocol::Jupiter));
        assert!(registry.program_id_to_parser.contains_key(&program_id));
    }

    #[test]
    fn test_event_parser_registry_add_parser_multiple_program_ids() {
        let mut registry = EventParserRegistry::default();
        let program_id_1 = Pubkey::new_unique();
        let program_id_2 = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id_1, program_id_2], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        assert_eq!(registry.parsers.len(), 1);
        assert_eq!(registry.program_id_to_parser.len(), 2);
        assert!(registry.program_id_to_parser.contains_key(&program_id_1));
        assert!(registry.program_id_to_parser.contains_key(&program_id_2));
    }

    #[test]
    fn test_event_parser_registry_get_parser_exists() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let retrieved_parser = registry.get_parser(&Protocol::Jupiter);
        assert!(retrieved_parser.is_some());
    }

    #[test]
    fn test_event_parser_registry_get_parser_not_exists() {
        let registry = EventParserRegistry::default();
        let retrieved_parser = registry.get_parser(&Protocol::Jupiter);
        assert!(retrieved_parser.is_none());
    }

    #[test]
    fn test_event_parser_registry_get_parser_for_program_exists() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let retrieved_parser = registry.get_parser_for_program(&program_id);
        assert!(retrieved_parser.is_some());
    }

    #[test]
    fn test_event_parser_registry_get_parser_for_program_not_exists() {
        let registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();

        let retrieved_parser = registry.get_parser_for_program(&program_id);
        assert!(retrieved_parser.is_none());
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_inner_instruction_with_events() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_inner_instruction(params);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_inner_instruction_no_events() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], false));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_inner_instruction(params);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_inner_instruction_empty_registry() {
        let registry = EventParserRegistry::default();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_inner_instruction(params);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_instruction_with_parser() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![program_id];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_instruction(params);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_instruction_no_parser() {
        let registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![program_id];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_instruction(params);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_parser_registry_parse_events_from_instruction_invalid_program_id_index() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let instruction = CompiledInstruction {
            program_id_index: 10, // Out of bounds
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![program_id];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_instruction(params);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_parser_registry_supported_program_ids_empty() {
        let registry = EventParserRegistry::default();
        let program_ids = registry.supported_program_ids();
        assert!(program_ids.is_empty());
    }

    #[test]
    fn test_event_parser_registry_supported_program_ids_with_parsers() {
        let mut registry = EventParserRegistry::default();
        let program_id_1 = Pubkey::new_unique();
        let program_id_2 = Pubkey::new_unique();
        let parser1 = Arc::new(MockEventParser::new(vec![program_id_1], true));
        let parser2 = Arc::new(MockEventParser::new(vec![program_id_2], true));

        registry.add_parser(Protocol::Jupiter, parser1);
        registry.add_parser(Protocol::MarginFi, parser2);

        let program_ids = registry.supported_program_ids();
        assert_eq!(program_ids.len(), 2);
        assert!(program_ids.contains(&program_id_1));
        assert!(program_ids.contains(&program_id_2));
    }

    #[test]
    fn test_event_parser_registry_should_handle_true() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        assert!(registry.should_handle(&program_id));
    }

    #[test]
    fn test_event_parser_registry_should_handle_false() {
        let registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();

        assert!(!registry.should_handle(&program_id));
    }

    #[test]
    fn test_event_parser_registry_with_all_parsers() {
        let registry = EventParserRegistry::with_all_parsers();
        // Should have all 9 protocol parsers
        assert_eq!(registry.parsers.len(), 9);
        // Should have program IDs mapped to parsers
        assert!(!registry.program_id_to_parser.is_empty());

        // Verify specific protocol parsers are present
        assert!(registry.parsers.contains_key(&Protocol::Bonk));
        assert!(registry.parsers.contains_key(&Protocol::Jupiter));
        assert!(registry.parsers.contains_key(&Protocol::MarginFi));
        assert!(registry.parsers.contains_key(&Protocol::MeteoraDlmm));
        assert!(registry.parsers.contains_key(&Protocol::OrcaWhirlpool));
        assert!(registry.parsers.contains_key(&Protocol::PumpSwap));
        assert!(registry.parsers.contains_key(&Protocol::RaydiumAmmV4));
        assert!(registry.parsers.contains_key(&Protocol::RaydiumClmm));
        assert!(registry.parsers.contains_key(&Protocol::RaydiumCpmm));
    }

    #[test]
    fn test_instruction_parse_params_debug() {
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![Pubkey::new_unique()];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("InstructionParseParams"));
    }

    #[test]
    fn test_inner_instruction_parse_params_debug() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("InnerInstructionParseParams"));
    }

    #[test]
    fn test_protocol_debug() {
        let protocol = Protocol::Jupiter;
        let debug_str = format!("{:?}", protocol);
        assert!(debug_str.contains("Jupiter"));
    }

    #[test]
    fn test_protocol_custom_debug() {
        let protocol = Protocol::Custom("TestProtocol".to_string());
        let debug_str = format!("{:?}", protocol);
        assert!(debug_str.contains("Custom"));
        assert!(debug_str.contains("TestProtocol"));
    }

    #[test]
    fn test_parse_events_from_instruction_none_block_time() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![program_id];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // Test with None block_time
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_instruction(params);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_events_from_inner_instruction_none_block_time() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_string(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // Test with None block_time
            program_received_time_ms: 1234567890123,
            index: "0".to_string(),
        };

        let events = registry.parse_events_from_inner_instruction(params);
        assert_eq!(events.len(), 1);
    }
}
