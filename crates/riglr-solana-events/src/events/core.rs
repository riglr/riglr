//! Core event parsing traits and utilities for Solana blockchain events.
//!
//! This module provides common types used across different protocol event parsers.

use serde::{Deserialize, Serialize};

/// Common event parameters shared across all protocol events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventParameters {
    /// Block timestamp in seconds
    pub block_time: i64,
    /// Block timestamp in milliseconds
    pub block_time_ms: i64,
    /// Unique identifier for the event
    pub id: String,
    /// Event index within the transaction
    pub index: String,
    /// Time when the program received the event in milliseconds
    pub program_received_time_ms: i64,
    /// Transaction signature
    pub signature: String,
    /// Solana slot number
    pub slot: u64,
}

impl EventParameters {
    /// Creates a new `EventParameters` instance with the provided values
    #[must_use]
    #[inline]
    pub const fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
    ) -> Self {
        Self {
            block_time,
            block_time_ms,
            id,
            index,
            program_received_time_ms,
            signature,
            slot,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use crate::{
        error::{Error, ParseResult},
        events::factory::{InnerInstructionParseParams, InstructionParseParams},
        events::parser_types::{GenericEventParseConfig, GenericEventParser},
        solana_metadata::SolanaEventMetadata,
        types::{EventType, ProtocolType},
    };
    use core::{any::Any, str::FromStr as _};
    use riglr_events_core::{error::EventResult, types::EventKind, Event, EventMetadata};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiCompiledInstruction;
    use std::time::SystemTime;

    // Mock event for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new(id: String) -> Self {
            let metadata =
                EventMetadata::new(id, EventKind::Custom("Mock".to_owned()), "test".to_owned());
            Self { metadata }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }

        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
            Ok(&mut self.metadata)
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
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

        fn timestamp(&self) -> SystemTime {
            SystemTime::now()
        }

        fn source(&self) -> &str {
            &self.metadata.source
        }

        fn matches_filter(&self, _filter: &dyn riglr_events_core::EventFilter) -> bool
        where
            Self: Sized,
        {
            true // Default implementation for mock
        }
    }

    // Mock parser functions
    #[allow(clippy::unnecessary_wraps)]
    fn mock_inner_instruction_parser(
        _data: &[u8],
        _metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Ok(Box::new(MockEvent::new("inner_test".to_owned())))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn mock_instruction_parser(
        _data: &[u8],
        _accounts: &[Pubkey],
        _metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Ok(Box::new(MockEvent::new("instruction_test".to_owned())))
    }

    fn mock_failing_inner_instruction_parser(
        _data: &[u8],
        _metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Err(Error::InvalidDataFormat("Mock failure".to_owned()))
    }

    fn mock_failing_instruction_parser(
        _data: &[u8],
        _accounts: &[Pubkey],
        _metadata: SolanaEventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        Err(Error::InvalidDataFormat("Mock failure".to_owned()))
    }

    #[test]
    fn generic_event_parse_config_creation() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");

        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3, 4],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        assert_eq!(config.program_id, program_id);
        assert_eq!(config.protocol_type, ProtocolType::OrcaWhirlpool);
        assert_eq!(config.inner_instruction_discriminator, "test_discriminator");
        assert_eq!(config.instruction_discriminator, &[1, 2, 3, 4]);
        assert_eq!(config.event_type, EventType::Swap);
    }

    #[test]
    fn generic_event_parse_config_clone() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");

        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "clone_test",
            instruction_discriminator: &[5, 6, 7, 8],
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let cloned_config = config.clone();
        assert_eq!(config.program_id, cloned_config.program_id);
        assert_eq!(config.protocol_type, cloned_config.protocol_type);
        assert_eq!(
            config.inner_instruction_discriminator,
            cloned_config.inner_instruction_discriminator
        );
        assert_eq!(
            config.instruction_discriminator,
            cloned_config.instruction_discriminator
        );
        assert_eq!(config.event_type, cloned_config.event_type);
    }

    #[test]
    fn generic_event_parse_config_debug() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");

        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::RaydiumAmm,
            inner_instruction_discriminator: "debug_test",
            instruction_discriminator: &[9, 10, 11, 12],
            event_type: EventType::RemoveLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let debug_string = format!("{config:?}");
        assert!(debug_string.contains("GenericEventParseConfig"));
        assert!(debug_string.contains("RaydiumAmm"));
        assert!(debug_string.contains("debug_test"));
        assert!(debug_string.contains("RemoveLiquidity"));
    }

    #[test]
    fn generic_event_parser_new_empty_configs() {
        let program_ids = vec![Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully")];
        let configs = vec![];

        let parser = GenericEventParser::new(program_ids.clone(), configs);

        assert_eq!(parser.program_ids, program_ids);
        assert!(parser.inner_instruction_configs.is_empty());
        assert!(parser.instruction_configs.is_empty());
    }

    #[test]
    fn generic_event_parser_new_single_config() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let program_ids = vec![program_id];

        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "single_test",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(program_ids.clone(), vec![config]);

        assert_eq!(parser.program_ids, program_ids);
        assert_eq!(parser.inner_instruction_configs.len(), 1);
        assert_eq!(parser.instruction_configs.len(), 1);
        assert!(parser.inner_instruction_configs.contains_key("single_test"));
        assert!(parser.instruction_configs.contains_key(&vec![1, 2, 3]));
    }

    #[test]
    fn generic_event_parser_new_multiple_configs_same_discriminator() {
        let first_program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let second_program_id = Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully");
        let program_ids = vec![first_program_id, second_program_id];

        let config1 = GenericEventParseConfig {
            program_id: first_program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "same_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let config2 = GenericEventParseConfig {
            program_id: second_program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "same_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(program_ids.clone(), vec![config1, config2]);

        assert_eq!(parser.program_ids, program_ids);
        assert_eq!(parser.inner_instruction_configs.len(), 1);
        assert_eq!(parser.instruction_configs.len(), 1);

        // Both configs should be grouped under the same discriminator
        let inner_configs = parser
            .inner_instruction_configs
            .get("same_discriminator")
            .expect("Expected inner instruction configs to be present");
        assert_eq!(inner_configs.len(), 2);

        let instruction_configs = parser
            .instruction_configs
            .get(&vec![1, 2, 3])
            .expect("Expected instruction configs to be present");
        assert_eq!(instruction_configs.len(), 2);
    }

    #[test]
    fn generic_event_parser_new_different_discriminators() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let program_ids = vec![program_id];

        let config1 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "discriminator1",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let config2 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "discriminator2",
            instruction_discriminator: &[4, 5, 6],
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(program_ids.clone(), vec![config1, config2]);

        assert_eq!(parser.program_ids, program_ids);
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

    #[test]
    fn generic_event_parser_inner_instruction_configs() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_config",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);
        let configs = parser.inner_instruction_configs();

        assert_eq!(configs.len(), 1);
        assert!(configs.contains_key("test_config"));
    }

    #[test]
    fn generic_event_parser_instruction_configs() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_config",
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
    fn generic_event_parser_should_handle_valid_program_id() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let parser = GenericEventParser::new(vec![program_id], vec![]);

        assert!(parser.should_handle(&program_id));
    }

    #[test]
    fn generic_event_parser_should_handle_invalid_program_id() {
        let first_program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let second_program_id = Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully");
        let parser = GenericEventParser::new(vec![first_program_id], vec![]);

        assert!(!parser.should_handle(&second_program_id));
    }

    #[test]
    fn generic_event_parser_should_handle_empty_program_ids() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let parser = GenericEventParser::new(vec![], vec![]);

        assert!(!parser.should_handle(&program_id));
    }

    #[test]
    fn generic_event_parser_supported_program_ids() {
        let first_program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let second_program_id = Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully");
        let program_ids = vec![first_program_id, second_program_id];
        let parser = GenericEventParser::new(program_ids.clone(), vec![]);

        let supported = parser.supported_program_ids();
        assert_eq!(supported, program_ids);
    }

    #[test]
    fn generic_event_parser_supported_program_ids_empty() {
        let parser = GenericEventParser::new(vec![], vec![]);
        let supported = parser.supported_program_ids();
        assert!(supported.is_empty());
    }

    #[test]
    fn generic_event_parser_parse_events_from_inner_instruction_valid_data() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        // Create valid base58 encoded data
        let test_data = vec![1, 2, 3, 4, 5];
        let encoded_data = bs58::encode(&test_data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: encoded_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
        let first_event = events.first().expect("Expected at least one event");
        assert_eq!(first_event.id(), "inner_test");
    }

    #[test]
    fn generic_event_parser_parse_events_from_inner_instruction_invalid_base58() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
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
            data: "invalid_base58_data_with_invalid_chars_0OIl".to_owned(),
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn generic_event_parser_parse_events_from_inner_instruction_parser_returns_none() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_failing_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let test_data = vec![1, 2, 3, 4, 5];
        let encoded_data = bs58::encode(&test_data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: encoded_data,
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn generic_event_parser_parse_events_from_inner_instruction_no_block_time() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
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
            stack_height: None,
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // No block time
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "inner_test"
        );
    }

    #[test]
    fn generic_event_parser_parse_events_from_instruction_matching_data() {
        static INSTRUCTION_DATA: &[u8] = &[1, 2, 3, 4];
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: INSTRUCTION_DATA.to_vec(),
        };

        let accounts = vec![
            Pubkey::from_str("11111111111111111111111111111113")
                .expect("Valid test pubkey should parse successfully"),
            Pubkey::from_str("11111111111111111111111111111114")
                .expect("Valid test pubkey should parse successfully"),
            Pubkey::from_str("11111111111111111111111111111115")
                .expect("Valid test pubkey should parse successfully"),
        ];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "instruction_test"
        );
    }

    #[test]
    fn generic_event_parser_parse_events_from_instruction_non_matching_data() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[1, 2, 3, 4],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![5, 6, 7, 8], // Different data
        };

        let accounts = vec![Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully")];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn generic_event_parser_parse_events_from_instruction_parser_returns_none() {
        static INSTRUCTION_DATA: &[u8] = &[1, 2, 3, 4];
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_failing_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: INSTRUCTION_DATA.to_vec(),
        };

        let accounts = vec![Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully")];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn generic_event_parser_parse_events_from_instruction_no_block_time() {
        static INSTRUCTION_DATA: &[u8] = &[1, 2, 3, 4];
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: INSTRUCTION_DATA.to_vec(),
        };

        let accounts = vec![Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully")];

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: None, // No block time
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "instruction_test"
        );
    }

    #[test]
    fn generic_event_parser_parse_events_from_instruction_empty_accounts() {
        static INSTRUCTION_DATA: &[u8] = &[1, 2, 3, 4];
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: INSTRUCTION_DATA.to_vec(),
        };

        let accounts = vec![]; // Empty accounts

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "instruction_test"
        );
    }

    #[test]
    fn generic_event_parser_parse_events_multiple_configs_same_instruction() {
        static INSTRUCTION_DATA: &[u8] = &[1, 2, 3, 4];
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");

        let config1 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator1",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let config2 = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::Jupiter,
            inner_instruction_discriminator: "test_discriminator2",
            instruction_discriminator: INSTRUCTION_DATA,
            event_type: EventType::AddLiquidity,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config1, config2]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: INSTRUCTION_DATA.to_vec(),
        };

        let accounts = vec![Pubkey::from_str("11111111111111111111111111111113")
            .expect("Valid test pubkey should parse successfully")];

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        // Should create events from both configs
        assert_eq!(events.len(), 2);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "instruction_test"
        );
        assert_eq!(
            events.get(1).expect("Expected second event").id(),
            "instruction_test"
        );
    }

    #[test]
    fn generic_event_parser_edge_case_empty_instruction_data() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
            inner_instruction_discriminator: "test_discriminator",
            instruction_discriminator: &[],
            event_type: EventType::Swap,
            inner_instruction_parser: mock_inner_instruction_parser,
            instruction_parser: mock_instruction_parser,
        };

        let parser = GenericEventParser::new(vec![program_id], vec![config]);

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![], // Empty data
        };

        let params = InstructionParseParams {
            instruction: &instruction,
            accounts: &[],
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "instruction_test"
        );
    }

    #[test]
    fn generic_event_parser_edge_case_large_slot_number() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
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
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: u64::MAX, // Maximum slot number
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "0".to_owned(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "inner_test"
        );
    }

    #[test]
    fn generic_event_parser_edge_case_special_index_string() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112")
            .expect("Valid test pubkey should parse successfully");
        let config = GenericEventParseConfig {
            program_id,
            protocol_type: ProtocolType::OrcaWhirlpool,
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
            stack_height: None,
        };

        let params = InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_signature",
            slot: 12345,
            block_time: Some(1_640_995_200),
            program_received_time_ms: 1_640_995_200_000,
            index: "special_index_!@#$%^&*()".to_owned(), // Special characters in index
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events.first().expect("Expected at least one event").id(),
            "inner_test"
        );
    }
}
