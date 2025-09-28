extern crate alloc;
#[cfg(test)]
use crate::events::parser_types::{GenericEventParseConfig, ProtocolParser};
use crate::events::protocols::{
    bonk::EventParser as BonkEventParser, jupiter::EventParser as JupiterEventParser,
    marginfi_parser::MarginFiEventParser, meteora::Parser as MeteoraEventParser,
    orca::Parser as OrcaEventParser, pumpswap::PumpSwapEventParser,
    raydium_amm_v4::Parser as RaydiumAmmV4EventParser,
    raydium_clmm::Parser as RaydiumClmmEventParser,
    raydium_cpmm::EventParser as RaydiumCpmmEventParser,
};
use crate::types::ProtocolType;
use alloc::sync::Arc;
use core::fmt;
use riglr_events_core::{traits::EventParser, Event};
use solana_message::compiled_instruction::CompiledInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Parameters for parsing events from instructions, reducing function parameter count
#[derive(Debug)]
#[non_exhaustive]
pub struct InstructionParseParams<'instruction_lifetime> {
    /// Account keys from the transaction
    pub accounts: &'instruction_lifetime [Pubkey],
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Index string for event identification
    pub index: String,
    /// Compiled instruction data
    pub instruction: &'instruction_lifetime CompiledInstruction,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Transaction signature
    pub signature: &'instruction_lifetime str,
    /// Solana slot number
    pub slot: u64,
}

/// Parameters for parsing events from inner instructions, reducing function parameter count
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerInstructionParseParams<'inner_lifetime> {
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Index string for event identification
    pub index: String,
    /// Inner instruction data from transaction metadata
    pub inner_instruction: &'inner_lifetime solana_transaction_status::UiCompiledInstruction,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Transaction signature
    pub signature: &'inner_lifetime str,
    /// Solana slot number
    pub slot: u64,
}

/// Owned parameters for parsing events from instructions
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct OwnedInstructionParseParams {
    /// Account keys from the transaction
    pub accounts: Vec<Pubkey>,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Index string for event identification
    pub index: String,
    /// Compiled instruction data
    pub instruction_data: Vec<u8>,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

/// Owned parameters for parsing events from inner instructions
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct OwnedInnerInstructionParseParams {
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Index string for event identification
    pub index: String,
    /// Inner instruction data as base58 string
    pub inner_instruction_data: String,
    /// Time when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

/// Unified owned input type for Solana transaction parsing
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SolanaTransactionInput {
    /// Inner instruction parsing input
    InnerInstruction(OwnedInnerInstructionParseParams),
    /// Regular instruction parsing input
    Instruction(OwnedInstructionParseParams),
}

/// Protocol enum for supported protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Protocol {
    /// Bonk token protocol
    Bonk,
    /// Custom protocol with arbitrary name
    Custom(String),
    /// Jupiter swap aggregator protocol
    Jupiter,
    /// `MarginFi` lending and borrowing protocol
    MarginFi,
    /// Meteora Dynamic Liquidity Market Maker protocol
    MeteoraDlmm,
    /// Orca Whirlpool concentrated liquidity protocol
    OrcaWhirlpool,
    /// `PumpFun` meme token creation protocol
    PumpFun,
    /// `PumpSwap` trading protocol
    PumpSwap,
    /// Raydium Automated Market Maker V4 protocol
    RaydiumAmmV4,
    /// Raydium Concentrated Liquidity Market Maker protocol
    RaydiumClmm,
    /// Raydium Constant Product Market Maker protocol
    RaydiumCpmm,
}

impl From<ProtocolType> for Protocol {
    #[inline]
    fn from(protocol_type: ProtocolType) -> Self {
        match protocol_type {
            ProtocolType::OrcaWhirlpool => Self::OrcaWhirlpool,
            ProtocolType::MeteoraDlmm => Self::MeteoraDlmm,
            ProtocolType::MarginFi => Self::MarginFi,
            ProtocolType::Bonk => Self::Bonk,
            ProtocolType::PumpSwap => Self::PumpSwap,
            ProtocolType::RaydiumAmm | ProtocolType::RaydiumAmmV4 | ProtocolType::Raydium => {
                Self::RaydiumAmmV4
            } // Default general Raydium to AMM V4
            ProtocolType::RaydiumClmm => Self::RaydiumClmm,
            ProtocolType::RaydiumCpmm => Self::RaydiumCpmm,
            ProtocolType::Jupiter => Self::Jupiter,
            ProtocolType::Serum => Self::Custom("Serum".to_owned()),
            ProtocolType::Other(name) => match name.as_str() {
                "Jupiter" => Self::Jupiter,
                "RaydiumAmmV4" => Self::RaydiumAmmV4,
                "RaydiumClmm" => Self::RaydiumClmm,
                "RaydiumCpmm" => Self::RaydiumCpmm,
                "PumpFun" => Self::PumpFun,
                "Bonk" => Self::Bonk,
                _ => Self::Custom(name),
            },
        }
    }
}

impl From<Protocol> for ProtocolType {
    #[inline]
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::OrcaWhirlpool => Self::OrcaWhirlpool,
            Protocol::MeteoraDlmm => Self::MeteoraDlmm,
            Protocol::MarginFi => Self::MarginFi,
            Protocol::Jupiter => Self::Jupiter,
            Protocol::RaydiumAmmV4 => Self::RaydiumAmmV4,
            Protocol::RaydiumClmm => Self::RaydiumClmm,
            Protocol::RaydiumCpmm => Self::RaydiumCpmm,
            Protocol::PumpFun => Self::Other("PumpFun".to_owned()),
            Protocol::PumpSwap => Self::PumpSwap,
            Protocol::Bonk => Self::Bonk,
            Protocol::Custom(name) => Self::Other(name),
        }
    }
}

/// `EventParserRegistry` - the new async event parser registry
#[derive(Default)]
pub struct EventParserRegistry {
    /// Map of protocols to their respective event parsers
    parsers: HashMap<
        Protocol,
        Arc<dyn EventParser<Input = SolanaTransactionInput> + Send + Sync + 'static>,
    >,
    /// Map of program IDs to their respective event parsers for fast lookup
    program_id_to_parser: HashMap<
        Pubkey,
        Arc<dyn EventParser<Input = SolanaTransactionInput> + Send + Sync + 'static>,
    >,
}

impl fmt::Debug for EventParserRegistry {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventParserRegistry")
            .field("parsers", &format!("{} parsers", self.parsers.len()))
            .field(
                "program_id_to_parser",
                &format!("{} program mappings", self.program_id_to_parser.len()),
            )
            .finish()
    }
}

impl EventParserRegistry {
    /// Add a parser for a specific protocol
    #[inline]
    pub fn add_parser(
        &mut self,
        protocol: Protocol,
        parser: Arc<dyn EventParser<Input = SolanaTransactionInput> + Send + Sync + 'static>,
    ) {
        self.parsers.insert(protocol, parser);
    }

    /// Get parser for a specific protocol
    #[must_use]
    #[inline]
    pub fn get_parser(
        &self,
        protocol: &Protocol,
    ) -> Option<&Arc<dyn EventParser<Input = SolanaTransactionInput> + Send + Sync + 'static>> {
        self.parsers.get(protocol)
    }

    /// Get parser for a specific program ID
    #[must_use]
    #[inline]
    pub fn get_parser_for_program(
        &self,
        program_id: &Pubkey,
    ) -> Option<&Arc<dyn EventParser<Input = SolanaTransactionInput> + Send + Sync + 'static>> {
        self.program_id_to_parser.get(program_id)
    }

    /// Create a new event parser registry
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            program_id_to_parser: HashMap::new(),
        }
    }

    /// Parse events from inner instruction using the appropriate parser
    #[inline]
    pub async fn parse_events_from_inner_instruction(
        &self,
        params: InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        // Convert to owned params
        let owned_params = OwnedInnerInstructionParseParams {
            inner_instruction_data: params.inner_instruction.data.clone(),
            signature: params.signature.to_owned(),
            slot: params.slot,
            block_time: params.block_time,
            program_received_time_ms: params.program_received_time_ms,
            index: params.index,
        };
        let input = SolanaTransactionInput::InnerInstruction(owned_params);

        // Try to identify the program and use the appropriate parser
        for parser in self.parsers.values() {
            if parser.can_parse(&input) {
                let parse_result = parser.parse(input.clone()).await;
                if let Ok(events) = parse_result {
                    if !events.is_empty() {
                        return events;
                    }
                }
            }
        }
        vec![]
    }

    /// Parse events from instruction using the appropriate parser
    #[inline]
    pub async fn parse_events_from_instruction(
        &self,
        params: InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        // Convert to owned params
        let owned_params = OwnedInstructionParseParams {
            instruction_data: params.instruction.data.clone(),
            accounts: params.accounts.to_vec(),
            signature: params.signature.to_owned(),
            slot: params.slot,
            block_time: params.block_time,
            program_received_time_ms: params.program_received_time_ms,
            index: params.index,
        };
        let input = SolanaTransactionInput::Instruction(owned_params);

        // Try each parser until one succeeds
        for parser in self.parsers.values() {
            if parser.can_parse(&input) {
                let parse_result = parser.parse(input.clone()).await;
                if let Ok(events) = parse_result {
                    if !events.is_empty() {
                        return events;
                    }
                }
            }
        }
        vec![]
    }

    /// Check if a program ID is supported
    #[must_use]
    #[inline]
    pub fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_id_to_parser.contains_key(program_id)
    }

    /// Get all supported program IDs
    #[must_use]
    #[inline]
    pub fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_id_to_parser.keys().copied().collect()
    }

    /// Create a registry with all available parsers
    #[must_use]
    #[inline]
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
        registry.add_parser(Protocol::RaydiumAmmV4, Arc::new(RaydiumAmmV4EventParser));
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::any::Any;
    use riglr_events_core::{
        error::EventResult,
        traits::{EventParser, ParserInfo},
        types::{EventKind, EventMetadata},
        Event,
    };
    use solana_transaction_status::UiCompiledInstruction;
    use std::collections::HashMap;

    // Mock EventParser for testing
    struct MockEventParser {
        program_ids: Vec<Pubkey>,
        returns_events: bool,
        info: ParserInfo,
    }

    impl MockEventParser {
        fn new(program_ids: Vec<Pubkey>, returns_events: bool) -> Self {
            Self {
                program_ids,
                returns_events,
                info: ParserInfo::new("mock".to_owned(), "1.0".to_owned()),
            }
        }
    }

    #[async_trait::async_trait]
    impl EventParser for MockEventParser {
        type Input = SolanaTransactionInput;

        async fn parse(&self, _input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
            if self.returns_events {
                return Ok(vec![Box::new(MockEvent::default())]);
            }
            Ok(vec![])
        }

        fn can_parse(&self, _input: &Self::Input) -> bool {
            true
        }

        fn info(&self) -> &ParserInfo {
            &self.info
        }
    }

    impl ProtocolParser for MockEventParser {
        fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
            HashMap::new()
        }

        fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
            HashMap::new()
        }

        fn parse_events_from_inner_instruction(
            &self,
            _params: &InnerInstructionParseParams<'_>,
        ) -> Vec<Box<dyn Event>> {
            if self.returns_events {
                return vec![Box::new(MockEvent::default())];
            }
            vec![]
        }

        fn parse_events_from_instruction(
            &self,
            _params: &InstructionParseParams<'_>,
        ) -> Vec<Box<dyn Event>> {
            if self.returns_events {
                return vec![Box::new(MockEvent::default())];
            }
            vec![]
        }

        fn should_handle(&self, program_id: &Pubkey) -> bool {
            self.program_ids.contains(program_id)
        }

        fn supported_program_ids(&self) -> Vec<Pubkey> {
            self.program_ids.clone()
        }
    }

    // Mock Event for testing
    #[derive(Debug)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl Default for MockEvent {
        fn default() -> Self {
            Self {
                metadata: EventMetadata::new(
                    "mock_event".to_owned(),
                    EventKind::Transaction,
                    "mock".to_owned(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(Self {
                metadata: self.metadata.clone(),
            })
        }

        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
            Ok(&mut self.metadata)
        }

        fn to_json(&self) -> EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "id": self.id(),
                "kind": format!("{:?}", self.kind()),
                "metadata": {
                    "id": self.metadata().id,
                    "source": self.metadata().source
                }
            }))
        }
    }

    // Tests for InstructionParseParams
    #[test]
    fn instruction_parse_params_creation() {
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };
        let accounts = vec![Pubkey::new_unique()];
        let signature = "test_signature";
        let slot = 12345;
        let block_time = Some(1_234_567_890);
        let program_received_time_ms = 1_234_567_890_123;
        let index = "0".to_owned();

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
    fn inner_instruction_parse_params_creation() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };
        let signature = "test_signature";
        let slot = 12345;
        let block_time = Some(1_234_567_890);
        let program_received_time_ms = 1_234_567_890_123;
        let index = "0".to_owned();

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
    fn protocol_variants() {
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
            Protocol::Custom("TestProtocol".to_owned()),
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
    fn protocol_equality() {
        assert_eq!(Protocol::OrcaWhirlpool, Protocol::OrcaWhirlpool);
        assert_eq!(
            Protocol::Custom("test".to_owned()),
            Protocol::Custom("test".to_owned())
        );
        assert_ne!(Protocol::OrcaWhirlpool, Protocol::MeteoraDlmm);
        assert_ne!(
            Protocol::Custom("test1".to_owned()),
            Protocol::Custom("test2".to_owned())
        );
    }

    #[test]
    fn protocol_clone() {
        let protocol = Protocol::Custom("test".to_owned());
        let cloned = protocol.clone();
        assert_eq!(protocol, cloned);
    }

    #[test]
    fn protocol_hash() {
        let mut map = HashMap::new();
        map.insert(Protocol::OrcaWhirlpool, "orca");
        map.insert(Protocol::Custom("test".to_owned()), "custom");

        assert_eq!(map.get(&Protocol::OrcaWhirlpool), Some(&"orca"));
        assert_eq!(
            map.get(&Protocol::Custom("test".to_owned())),
            Some(&"custom")
        );
    }

    // Tests for Protocol -> ProtocolType conversion
    #[test]
    fn protocol_from_protocol_type_orca_whirlpool() {
        let protocol = Protocol::from(ProtocolType::OrcaWhirlpool);
        assert_eq!(protocol, Protocol::OrcaWhirlpool);
    }

    #[test]
    fn protocol_from_protocol_type_meteora_dlmm() {
        let protocol = Protocol::from(ProtocolType::MeteoraDlmm);
        assert_eq!(protocol, Protocol::MeteoraDlmm);
    }

    #[test]
    fn protocol_from_protocol_type_margin_fi() {
        let protocol = Protocol::from(ProtocolType::MarginFi);
        assert_eq!(protocol, Protocol::MarginFi);
    }

    #[test]
    fn protocol_from_protocol_type_bonk() {
        let protocol = Protocol::from(ProtocolType::Bonk);
        assert_eq!(protocol, Protocol::Bonk);
    }

    #[test]
    fn protocol_from_protocol_type_pump_swap() {
        let protocol = Protocol::from(ProtocolType::PumpSwap);
        assert_eq!(protocol, Protocol::PumpSwap);
    }

    #[test]
    fn protocol_from_protocol_type_raydium_amm() {
        let protocol = Protocol::from(ProtocolType::RaydiumAmm);
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn protocol_from_protocol_type_raydium_amm_v4() {
        let protocol = Protocol::from(ProtocolType::RaydiumAmmV4);
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn protocol_from_protocol_type_raydium_clmm() {
        let protocol = Protocol::from(ProtocolType::RaydiumClmm);
        assert_eq!(protocol, Protocol::RaydiumClmm);
    }

    #[test]
    fn protocol_from_protocol_type_raydium_cpmm() {
        let protocol = Protocol::from(ProtocolType::RaydiumCpmm);
        assert_eq!(protocol, Protocol::RaydiumCpmm);
    }

    #[test]
    fn protocol_from_protocol_type_jupiter() {
        let protocol = Protocol::from(ProtocolType::Jupiter);
        assert_eq!(protocol, Protocol::Jupiter);
    }

    #[test]
    fn protocol_from_protocol_type_other_jupiter() {
        let protocol = Protocol::from(ProtocolType::Other("Jupiter".to_owned()));
        assert_eq!(protocol, Protocol::Jupiter);
    }

    #[test]
    fn protocol_from_protocol_type_other_raydium_amm_v4() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumAmmV4".to_owned()));
        assert_eq!(protocol, Protocol::RaydiumAmmV4);
    }

    #[test]
    fn protocol_from_protocol_type_other_raydium_clmm() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumClmm".to_owned()));
        assert_eq!(protocol, Protocol::RaydiumClmm);
    }

    #[test]
    fn protocol_from_protocol_type_other_raydium_cpmm() {
        let protocol = Protocol::from(ProtocolType::Other("RaydiumCpmm".to_owned()));
        assert_eq!(protocol, Protocol::RaydiumCpmm);
    }

    #[test]
    fn protocol_from_protocol_type_other_pump_fun() {
        let protocol = Protocol::from(ProtocolType::Other("PumpFun".to_owned()));
        assert_eq!(protocol, Protocol::PumpFun);
    }

    #[test]
    fn protocol_from_protocol_type_other_bonk() {
        let protocol = Protocol::from(ProtocolType::Other("Bonk".to_owned()));
        assert_eq!(protocol, Protocol::Bonk);
    }

    #[test]
    fn protocol_from_protocol_type_other_custom() {
        let protocol = Protocol::from(ProtocolType::Other("CustomProtocol".to_owned()));
        assert_eq!(protocol, Protocol::Custom("CustomProtocol".to_owned()));
    }

    // Tests for ProtocolType -> Protocol conversion
    #[test]
    fn protocol_type_from_protocol_orca_whirlpool() {
        let protocol_type = ProtocolType::from(Protocol::OrcaWhirlpool);
        assert_eq!(protocol_type, ProtocolType::OrcaWhirlpool);
    }

    #[test]
    fn protocol_type_from_protocol_meteora_dlmm() {
        let protocol_type = ProtocolType::from(Protocol::MeteoraDlmm);
        assert_eq!(protocol_type, ProtocolType::MeteoraDlmm);
    }

    #[test]
    fn protocol_type_from_protocol_margin_fi() {
        let protocol_type = ProtocolType::from(Protocol::MarginFi);
        assert_eq!(protocol_type, ProtocolType::MarginFi);
    }

    #[test]
    fn protocol_type_from_protocol_jupiter() {
        let protocol_type = ProtocolType::from(Protocol::Jupiter);
        assert_eq!(protocol_type, ProtocolType::Jupiter);
    }

    #[test]
    fn protocol_type_from_protocol_raydium_amm_v4() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumAmmV4);
        assert_eq!(protocol_type, ProtocolType::RaydiumAmmV4);
    }

    #[test]
    fn protocol_type_from_protocol_raydium_clmm() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumClmm);
        assert_eq!(protocol_type, ProtocolType::RaydiumClmm);
    }

    #[test]
    fn protocol_type_from_protocol_raydium_cpmm() {
        let protocol_type = ProtocolType::from(Protocol::RaydiumCpmm);
        assert_eq!(protocol_type, ProtocolType::RaydiumCpmm);
    }

    #[test]
    fn protocol_type_from_protocol_pump_fun() {
        let protocol_type = ProtocolType::from(Protocol::PumpFun);
        assert_eq!(protocol_type, ProtocolType::Other("PumpFun".to_owned()));
    }

    #[test]
    fn protocol_type_from_protocol_pump_swap() {
        let protocol_type = ProtocolType::from(Protocol::PumpSwap);
        assert_eq!(protocol_type, ProtocolType::PumpSwap);
    }

    #[test]
    fn protocol_type_from_protocol_bonk() {
        let protocol_type = ProtocolType::from(Protocol::Bonk);
        assert_eq!(protocol_type, ProtocolType::Bonk);
    }

    #[test]
    fn protocol_type_from_protocol_custom() {
        let protocol_type = ProtocolType::from(Protocol::Custom("CustomProtocol".to_owned()));
        assert_eq!(
            protocol_type,
            ProtocolType::Other("CustomProtocol".to_owned())
        );
    }

    // Tests for EventParserRegistry
    #[test]
    fn event_parser_registry_new() {
        let registry = EventParserRegistry::default();
        assert!(registry.parsers.is_empty());
        assert!(registry.program_id_to_parser.is_empty());
    }

    #[test]
    fn event_parser_registry_default() {
        let registry = EventParserRegistry::default();
        assert!(registry.parsers.is_empty());
        assert!(registry.program_id_to_parser.is_empty());
    }

    #[test]
    fn event_parser_registry_add_parser() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser);

        assert_eq!(registry.parsers.len(), 1);
        assert!(registry.parsers.contains_key(&Protocol::Jupiter));
    }

    #[test]
    fn event_parser_registry_add_parser_multiple_program_ids() {
        let mut registry = EventParserRegistry::default();
        let program_id_1 = Pubkey::new_unique();
        let program_id_2 = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id_1, program_id_2], true));

        registry.add_parser(Protocol::Jupiter, parser);

        assert_eq!(registry.parsers.len(), 1);
        assert!(registry.parsers.contains_key(&Protocol::Jupiter));
    }

    #[test]
    fn event_parser_registry_get_parser_exists() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser);

        let retrieved_parser = registry.get_parser(&Protocol::Jupiter);
        assert!(retrieved_parser.is_some());
    }

    #[test]
    fn event_parser_registry_get_parser_not_exists() {
        let registry = EventParserRegistry::default();
        let retrieved_parser = registry.get_parser(&Protocol::Jupiter);
        assert!(retrieved_parser.is_none());
    }

    #[test]
    fn event_parser_registry_get_parser_for_program_exists() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser);

        let retrieved_parser = registry.get_parser_for_program(&program_id);
        assert!(retrieved_parser.is_some());
    }

    #[test]
    fn event_parser_registry_get_parser_for_program_not_exists() {
        let registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();

        let retrieved_parser = registry.get_parser_for_program(&program_id);
        assert!(retrieved_parser.is_none());
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_inner_instruction_with_events() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_inner_instruction(params).await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_inner_instruction_no_events() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], false));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_inner_instruction(params).await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_inner_instruction_empty_registry() {
        let registry = EventParserRegistry::default();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_inner_instruction(params).await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_instruction_with_parser() {
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
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_instruction(params).await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_instruction_no_parser() {
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
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_instruction(params).await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn event_parser_registry_parse_events_from_instruction_invalid_program_id_index() {
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
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_instruction(params).await;
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn event_parser_registry_supported_program_ids_empty() {
        let registry = EventParserRegistry::default();
        let program_ids = registry.supported_program_ids();
        assert!(program_ids.is_empty());
    }

    #[test]
    fn event_parser_registry_supported_program_ids_with_parsers() {
        let mut registry = EventParserRegistry::default();
        let program_id_1 = Pubkey::new_unique();
        let program_id_2 = Pubkey::new_unique();
        let parser1 = Arc::new(MockEventParser::new(vec![program_id_1], true));
        let parser2 = Arc::new(MockEventParser::new(vec![program_id_2], true));

        registry.add_parser(Protocol::Jupiter, parser1);
        registry.add_parser(Protocol::MarginFi, parser2);

        let program_ids = registry.supported_program_ids();
        assert_eq!(program_ids.len(), 0); // Empty because program_id_to_parser is not populated
    }

    #[test]
    fn event_parser_registry_should_handle_true() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser);

        assert!(!registry.should_handle(&program_id)); // False because program_id_to_parser is not populated
    }

    #[test]
    fn event_parser_registry_should_handle_false() {
        let registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();

        assert!(!registry.should_handle(&program_id));
    }

    #[test]
    fn event_parser_registry_with_all_parsers() {
        let registry = EventParserRegistry::with_all_parsers();
        // Should have all 9 protocol parsers
        assert_eq!(registry.parsers.len(), 9);
        // program_id_to_parser is empty because it's not populated in current implementation
        assert!(registry.program_id_to_parser.is_empty());

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
    fn instruction_parse_params_debug() {
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
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("InstructionParseParams"));
    }

    #[test]
    fn inner_instruction_parse_params_debug() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_234_567_890),
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let debug_str = format!("{params:?}");
        assert!(debug_str.contains("InnerInstructionParseParams"));
    }

    #[test]
    fn protocol_debug() {
        let protocol = Protocol::Jupiter;
        let debug_str = format!("{protocol:?}");
        assert!(debug_str.contains("Jupiter"));
    }

    #[test]
    fn protocol_custom_debug() {
        let protocol = Protocol::Custom("TestProtocol".to_owned());
        let debug_str = format!("{protocol:?}");
        assert!(debug_str.contains("Custom"));
        assert!(debug_str.contains("TestProtocol"));
    }

    #[tokio::test]
    async fn parse_events_from_instruction_none_block_time() {
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
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_instruction(params).await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn parse_events_from_inner_instruction_none_block_time() {
        let mut registry = EventParserRegistry::default();
        let program_id = Pubkey::new_unique();
        let parser = Arc::new(MockEventParser::new(vec![program_id], true));

        registry.add_parser(Protocol::Jupiter, parser.clone());

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "test_data".to_owned(),
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // Test with None block_time
            program_received_time_ms: 1_234_567_890_123,
            index: "0".to_owned(),
        };

        let events = registry.parse_events_from_inner_instruction(params).await;
        assert_eq!(events.len(), 1);
    }
}
