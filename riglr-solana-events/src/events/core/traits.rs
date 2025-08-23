use crate::error::ParseResult;
use crate::metadata_helpers;
use crate::types::{EventMetadata, EventType, ProtocolType};
use riglr_events_core::Event;
use std::fmt::Debug;

// UnifiedEvent trait has been removed. Use riglr_events_core::Event instead.

/// Event Parser trait - defines the core methods for event parsing
/// This now returns Event trait objects instead of UnifiedEvent
#[async_trait::async_trait]
pub trait EventParser: Send + Sync {
    /// Get inner instruction parsing configurations
    fn inner_instruction_configs(
        &self,
    ) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>;

    /// Get instruction parsing configurations
    fn instruction_configs(
        &self,
    ) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>;

    /// Parse event data from inner instruction
    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>>;

    /// Parse event data from instruction
    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
    ) -> Vec<Box<dyn Event>>;

    /// Check if this program ID should be handled
    fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool;

    /// Get supported program ID list
    fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey>;
}

/// Generic event parser configuration
#[derive(Debug, Clone)]
pub struct GenericEventParseConfig {
    /// Program ID this configuration applies to
    pub program_id: solana_sdk::pubkey::Pubkey,
    /// Protocol type for events generated from this configuration
    pub protocol_type: ProtocolType,
    /// Discriminator string for inner instructions
    pub inner_instruction_discriminator: &'static str,
    /// Discriminator bytes for instructions
    pub instruction_discriminator: &'static [u8],
    /// Type of events this configuration generates
    pub event_type: EventType,
    /// Parser function for inner instructions
    pub inner_instruction_parser: InnerInstructionEventParser,
    /// Parser function for instructions
    pub instruction_parser: InstructionEventParser,
}

/// Inner instruction event parser
pub type InnerInstructionEventParser =
    fn(data: &[u8], metadata: EventMetadata) -> ParseResult<Box<dyn Event>>;

/// Instruction event parser
pub type InstructionEventParser = fn(
    data: &[u8],
    accounts: &[solana_sdk::pubkey::Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>>;

/// Generic event parser base class
pub struct GenericEventParser {
    /// List of program IDs this parser handles
    pub program_ids: Vec<solana_sdk::pubkey::Pubkey>,
    /// Configuration mapping for inner instruction parsing by discriminator
    pub inner_instruction_configs:
        std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>,
    /// Configuration mapping for instruction parsing by discriminator bytes
    pub instruction_configs: std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

impl GenericEventParser {
    /// Create new generic event parser
    pub fn new(
        program_ids: Vec<solana_sdk::pubkey::Pubkey>,
        configs: Vec<GenericEventParseConfig>,
    ) -> Self {
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

        Self {
            program_ids,
            inner_instruction_configs,
            instruction_configs,
        }
    }
}

#[async_trait::async_trait]
impl EventParser for GenericEventParser {
    fn inner_instruction_configs(
        &self,
    ) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(
        &self,
    ) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let core_metadata = metadata_helpers::create_solana_metadata(
                        format!("{}_{}", params.signature, params.index),
                        riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                        "solana".to_string(),
                        params.slot,
                        Some(params.signature.to_string()),
                        Some(config.program_id),
                        params.index.parse().ok(),
                        params.block_time,
                        config.protocol_type.clone(),
                        config.event_type.clone(),
                    );

                    let metadata = crate::types::EventMetadata::new(
                        params.signature.to_string(),
                        params.slot,
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                        params.index.clone(),
                        params.program_received_time_ms,
                        core_metadata,
                    );

                    match (config.inner_instruction_parser)(&data, metadata) {
                        Ok(event) => events.push(event),
                        Err(_) => {
                            // Log the error or handle it as appropriate for your use case
                            // For now, we continue processing other configs
                        }
                    }
                }
            }
        }

        events
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        if let Some(configs) = self.instruction_configs.get(&params.instruction.data) {
            for config in configs {
                let core_metadata = metadata_helpers::create_solana_metadata(
                    format!("{}_{}", params.signature, params.index),
                    riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                    "solana".to_string(),
                    params.slot,
                    Some(params.signature.to_string()),
                    Some(config.program_id),
                    params.index.parse().ok(),
                    params.block_time,
                    config.protocol_type.clone(),
                    config.event_type.clone(),
                );

                let metadata = crate::types::EventMetadata::new(
                    params.signature.to_string(),
                    params.slot,
                    config.event_type.clone(),
                    config.protocol_type.clone(),
                    params.index.clone(),
                    params.program_received_time_ms,
                    core_metadata,
                );

                match (config.instruction_parser)(
                    &params.instruction.data,
                    params.accounts,
                    metadata,
                ) {
                    Ok(event) => events.push(event),
                    Err(_) => {
                        // Log the error or handle it as appropriate for your use case
                        // For now, we continue processing other configs
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::Event;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiCompiledInstruction;

    // Mock Event implementation for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
        kind: riglr_events_core::EventKind,
        metadata: riglr_events_core::EventMetadata,
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.id
        }

        fn kind(&self) -> &riglr_events_core::EventKind {
            &self.kind
        }

        fn metadata(&self) -> &riglr_events_core::EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
            &mut self.metadata
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    }

    // Helper function to create a mock event
    fn create_mock_event() -> Box<dyn Event> {
        Box::new(MockEvent {
            id: "test_event".to_string(),
            kind: riglr_events_core::EventKind::Transaction,
            metadata: riglr_events_core::EventMetadata::new(
                "test_event".to_string(),
                riglr_events_core::EventKind::Transaction,
                "test-source".to_string(),
            ),
        })
    }

    // Mock parser functions
    fn mock_inner_instruction_parser(
        _data: &[u8],
        _metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Ok(create_mock_event())
    }

    fn mock_instruction_parser(
        _data: &[u8],
        _accounts: &[Pubkey],
        _metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Ok(create_mock_event())
    }

    fn mock_inner_instruction_parser_none(
        _data: &[u8],
        _metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Err(crate::error::ParseError::Generic(
            "Mock parser returning error".to_string(),
        ))
    }

    fn mock_instruction_parser_none(
        _data: &[u8],
        _accounts: &[Pubkey],
        _metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Err(crate::error::ParseError::Generic(
            "Mock parser returning error".to_string(),
        ))
    }

    #[test]
    fn test_generic_event_parse_config_debug_clone() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GenericEventParseConfig"));

        // Test Clone trait
        let cloned_config = config.clone();
        assert_eq!(cloned_config.program_id, config.program_id);
        assert_eq!(cloned_config.protocol_type, config.protocol_type);
        assert_eq!(
            cloned_config.inner_instruction_discriminator,
            config.inner_instruction_discriminator
        );
        assert_eq!(
            cloned_config.instruction_discriminator,
            config.instruction_discriminator
        );
        assert_eq!(cloned_config.event_type, config.event_type);
    }

    #[test]
    fn test_generic_event_parser_new_empty_configs() {
        let program_ids = vec![Pubkey::new_unique()];
        let configs = vec![];

        let parser = GenericEventParser::new(program_ids.clone(), configs);

        assert_eq!(parser.program_ids, program_ids);
        assert!(parser.inner_instruction_configs.is_empty());
        assert!(parser.instruction_configs.is_empty());
    }

    #[test]
    fn test_generic_event_parser_new_single_config() {
        let program_id = Pubkey::new_unique();
        let program_ids = vec![program_id];

        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(program_ids.clone(), vec![config.clone()]);

        assert_eq!(parser.program_ids, program_ids);
        assert_eq!(parser.inner_instruction_configs.len(), 1);
        assert_eq!(parser.instruction_configs.len(), 1);

        // Check inner instruction config
        let inner_configs = parser
            .inner_instruction_configs
            .get("test_discriminator")
            .unwrap();
        assert_eq!(inner_configs.len(), 1);
        assert_eq!(inner_configs[0].program_id, program_id);

        // Check instruction config
        let inst_configs = parser.instruction_configs.get(&vec![1, 2, 3]).unwrap();
        assert_eq!(inst_configs.len(), 1);
        assert_eq!(inst_configs[0].program_id, program_id);
    }

    #[test]
    fn test_generic_event_parser_new_multiple_configs_same_discriminator() {
        let program_id1 = Pubkey::new_unique();
        let program_id2 = Pubkey::new_unique();
        let program_ids = vec![program_id1, program_id2];

        let config1 = GenericEventParseConfig {
            program_id: program_id1,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "same_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let config2 = GenericEventParseConfig {
            program_id: program_id2,
            protocol_type: ProtocolType::Raydium,
            inner_instruction_discriminator: "same_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(program_ids.clone(), vec![config1, config2]);

        assert_eq!(parser.program_ids, program_ids);

        // Check that both configs are stored under the same discriminator
        let inner_configs = parser
            .inner_instruction_configs
            .get("same_discriminator")
            .unwrap();
        assert_eq!(inner_configs.len(), 2);

        let inst_configs = parser.instruction_configs.get(&vec![1, 2, 3]).unwrap();
        assert_eq!(inst_configs.len(), 2);
    }

    #[test]
    fn test_generic_event_parser_inner_instruction_configs() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);
        let configs = parser.inner_instruction_configs();

        assert_eq!(configs.len(), 1);
        assert!(configs.contains_key("test_discriminator"));
    }

    #[test]
    fn test_generic_event_parser_instruction_configs() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);
        let configs = parser.instruction_configs();

        assert_eq!(configs.len(), 1);
        assert!(configs.contains_key(&vec![1, 2, 3]));
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_inner_instruction_valid_data() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        // Create a valid base58 encoded data
        let test_data = vec![1, 2, 3, 4, 5];
        let encoded_data = bs58::encode(&test_data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: encoded_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id(), "test_event");
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_inner_instruction_invalid_data() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: "invalid_base58_data_!@#$%".to_string(),
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_inner_instruction_none_block_time() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let test_data = vec![1, 2, 3, 4, 5];
        let encoded_data = bs58::encode(&test_data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: encoded_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 100,
            block_time: None, // None block_time
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_inner_instruction_parser_returns_none() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser_none,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let test_data = vec![1, 2, 3, 4, 5];
        let encoded_data = bs58::encode(&test_data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: encoded_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_instruction_matching_config() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = solana_sdk::instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3],
        };

        let accounts = vec![Pubkey::new_unique()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id(), "test_event");
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_instruction_no_matching_config() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = solana_sdk::instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![4, 5, 6], // Different data that doesn't match config
        };

        let accounts = vec![Pubkey::new_unique()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_instruction_none_block_time() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = solana_sdk::instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3],
        };

        let accounts = vec![Pubkey::new_unique()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 100,
            block_time: None, // None block_time
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_generic_event_parser_parse_events_from_instruction_parser_returns_none() {
        let program_id = Pubkey::new_unique();
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser_none,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = solana_sdk::instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![1, 2, 3],
        };

        let accounts = vec![Pubkey::new_unique()];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 100,
            block_time: Some(1234567890),
            program_received_time_ms: 1234567890000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert!(events.is_empty());
    }

    #[test]
    fn test_generic_event_parser_should_handle_true() {
        let program_id = Pubkey::new_unique();
        let parser = GenericEventParser::new(vec![program_id], vec![]);

        assert!(parser.should_handle(&program_id));
    }

    #[test]
    fn test_generic_event_parser_should_handle_false() {
        let program_id1 = Pubkey::new_unique();
        let program_id2 = Pubkey::new_unique();
        let parser = GenericEventParser::new(vec![program_id1], vec![]);

        assert!(!parser.should_handle(&program_id2));
    }

    #[test]
    fn test_generic_event_parser_should_handle_empty_program_ids() {
        let parser = GenericEventParser::new(vec![], vec![]);
        let program_id = Pubkey::new_unique();

        assert!(!parser.should_handle(&program_id));
    }

    #[test]
    fn test_generic_event_parser_supported_program_ids() {
        let program_id1 = Pubkey::new_unique();
        let program_id2 = Pubkey::new_unique();
        let program_ids = vec![program_id1, program_id2];
        let parser = GenericEventParser::new(program_ids.clone(), vec![]);

        let supported = parser.supported_program_ids();
        assert_eq!(supported, program_ids);
    }

    #[test]
    fn test_generic_event_parser_supported_program_ids_empty() {
        let parser = GenericEventParser::new(vec![], vec![]);

        let supported = parser.supported_program_ids();
        assert!(supported.is_empty());
    }

    #[test]
    fn test_generic_event_parser_multiple_configs_different_discriminators() {
        let program_id = Pubkey::new_unique();

        let config1 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "discriminator1",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let config2 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Raydium,
            inner_instruction_discriminator: "discriminator2",
            instruction_discriminator: &[4, 5, 6],
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config1, config2]);

        assert_eq!(parser.inner_instruction_configs.len(), 2);
        assert_eq!(parser.instruction_configs.len(), 2);

        assert!(parser
            .inner_instruction_configs
            .contains_key("discriminator1"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("discriminator2"));
        assert!(parser.instruction_configs.contains_key(&vec![1, 2, 3]));
        assert!(parser.instruction_configs.contains_key(&vec![4, 5, 6]));
    }
}
