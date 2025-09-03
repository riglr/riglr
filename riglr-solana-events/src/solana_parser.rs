//! Solana-specific event parser implementations using riglr-events-core.
//!
//! This module provides event parsers that implement the riglr-events-core EventParser trait
//! while leveraging the existing Solana parsing logic.

use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiCompiledInstruction;
use std::sync::Arc;

use crate::events::core::traits::EventParser as LegacyEventParser;
use crate::events::factory::{
    EventParserRegistry, InnerInstructionParseParams, InstructionParseParams, Protocol,
};
use crate::solana_events::SolanaEvent;
use riglr_events_core::prelude::*;

/// Input type for Solana transaction parsing
#[derive(Debug, Clone)]
pub struct SolanaTransactionInput {
    /// Solana instruction data
    pub instruction: solana_message::compiled_instruction::CompiledInstruction,
    /// Account keys from the transaction
    pub accounts: Vec<Pubkey>,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// When the event was received by the parser
    pub received_time: chrono::DateTime<chrono::Utc>,
    /// Instruction index for identification
    pub instruction_index: usize,
}

/// Input type for Solana inner instruction parsing
#[derive(Debug, Clone)]
pub struct SolanaInnerInstructionInput {
    /// Inner instruction data
    pub inner_instruction: UiCompiledInstruction,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// When the event was received by the parser
    pub received_time: chrono::DateTime<chrono::Utc>,
    /// Instruction index for identification
    pub instruction_index: String,
}

/// Solana event parser that bridges between legacy and new parsers
pub struct SolanaEventParser {
    /// Legacy multi-parser for actual parsing logic
    legacy_parser: EventParserRegistry,
    /// Parser information
    info: ParserInfo,
    /// Supported program IDs
    supported_programs: Vec<Pubkey>,
}

impl Default for SolanaEventParser {
    fn default() -> Self {
        let legacy_parser = EventParserRegistry::default();
        let supported_programs = legacy_parser.supported_program_ids();
        Self {
            legacy_parser,
            info: ParserInfo::new("solana-event-parser".to_string(), "1.0.0".to_string())
                .with_kind(riglr_events_core::types::EventKind::Transaction)
                .with_kind(riglr_events_core::types::EventKind::Swap)
                .with_kind(riglr_events_core::types::EventKind::Liquidity)
                .with_kind(riglr_events_core::types::EventKind::Transfer)
                .with_format("solana-instruction".to_string())
                .with_format("solana-inner-instruction".to_string()),
            supported_programs,
        }
    }
}

impl SolanaEventParser {
    /// Create a new Solana event parser
    pub fn new() -> Self {
        let legacy_parser = EventParserRegistry::default();
        let supported_programs = legacy_parser.supported_program_ids();
        Self {
            legacy_parser,
            info: ParserInfo::new("solana-event-parser".to_string(), "1.0.0".to_string())
                .with_kind(riglr_events_core::types::EventKind::Transaction)
                .with_kind(riglr_events_core::types::EventKind::Swap)
                .with_kind(riglr_events_core::types::EventKind::Liquidity)
                .with_kind(riglr_events_core::types::EventKind::Transfer)
                .with_format("solana-instruction".to_string())
                .with_format("solana-inner-instruction".to_string()),
            supported_programs,
        }
    }

    /// Create with specific legacy parser
    pub fn with_legacy_parser(legacy_parser: EventParserRegistry) -> Self {
        let supported_programs = legacy_parser.supported_program_ids();

        Self {
            legacy_parser,
            info: ParserInfo::new("solana-event-parser".to_string(), "1.0.0".to_string())
                .with_kind(riglr_events_core::types::EventKind::Transaction)
                .with_kind(riglr_events_core::types::EventKind::Swap)
                .with_kind(riglr_events_core::types::EventKind::Liquidity)
                .with_kind(riglr_events_core::types::EventKind::Transfer)
                .with_format("solana-instruction".to_string())
                .with_format("solana-inner-instruction".to_string()),
            supported_programs,
        }
    }

    /// Add a protocol parser
    pub fn add_protocol_parser<P>(
        &mut self,
        _protocol: Protocol,
        _parser: Arc<dyn LegacyEventParser>,
    ) where
        P: LegacyEventParser + 'static,
    {
        // Note: This would require modifying MultiEventParser to be mutable
        // For now, we'll focus on using pre-built parsers
    }

    /// Check if a program ID is supported
    pub fn supports_program(&self, program_id: &Pubkey) -> bool {
        self.supported_programs.contains(program_id)
    }

    /// Parse events from a Solana instruction
    pub async fn parse_instruction(
        &self,
        input: SolanaTransactionInput,
    ) -> EventResult<Vec<SolanaEvent>> {
        let legacy_events = self
            .legacy_parser
            .parse_events_from_instruction(InstructionParseParams {
                instruction: &input.instruction,
                accounts: &input.accounts,
                signature: &input.signature,
                slot: input.slot,
                block_time: input.block_time,
                program_received_time_ms: input.received_time.timestamp_millis(),
                index: input.instruction_index.to_string(),
            })
            .await;

        let solana_events = legacy_events
            .into_iter()
            .map(|event| self.convert_legacy_event(event, &input))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(solana_events)
    }

    /// Parse events from a Solana inner instruction
    pub async fn parse_inner_instruction(
        &self,
        input: SolanaInnerInstructionInput,
    ) -> EventResult<Vec<SolanaEvent>> {
        let legacy_events = self
            .legacy_parser
            .parse_events_from_inner_instruction(InnerInstructionParseParams {
                inner_instruction: &input.inner_instruction,
                signature: &input.signature,
                slot: input.slot,
                block_time: input.block_time,
                program_received_time_ms: input.received_time.timestamp_millis(),
                index: input.instruction_index.clone(),
            })
            .await;

        let solana_events = legacy_events
            .into_iter()
            .map(|event| self.convert_legacy_inner_event(event, &input))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(solana_events)
    }

    /// Convert a legacy Event to a SolanaEvent
    fn convert_legacy_event(
        &self,
        event: Box<dyn riglr_events_core::Event>,
        input: &SolanaTransactionInput,
    ) -> EventResult<SolanaEvent> {
        // Extract data using the event's as_any method for downcasting
        let event_data = serde_json::json!({
            "kind": event.kind().to_string(),
            "signature": input.signature.clone(),
            "slot": input.slot,
            "timestamp": input.received_time.timestamp_millis(),
        });

        // Create metadata for the new event system
        let metadata = crate::types::metadata_helpers::create_solana_metadata(
            event.id().to_string(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            crate::types::ProtocolType::Other("Solana".to_string()),
            crate::types::EventType::Swap,
            self.extract_program_id(&input.accounts, &input.instruction)?,
            input.instruction_index.to_string(),
            input.received_time.timestamp_millis(),
        );

        Ok(SolanaEvent::new(metadata, event_data))
    }

    /// Convert a legacy Event from inner instruction to a SolanaEvent
    fn convert_legacy_inner_event(
        &self,
        event: Box<dyn riglr_events_core::Event>,
        input: &SolanaInnerInstructionInput,
    ) -> EventResult<SolanaEvent> {
        let event_data = serde_json::json!({
            "kind": event.kind().to_string(),
            "signature": input.signature.clone(),
            "slot": input.slot,
            "timestamp": input.received_time.timestamp_millis(),
            "is_inner_instruction": true,
        });

        // For inner instructions, we may not have direct program ID access
        // Use a default program ID or try to extract from the event
        let program_id = Pubkey::default(); // This would need better logic in production

        let metadata = crate::types::metadata_helpers::create_solana_metadata(
            event.id().to_string(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            crate::types::ProtocolType::Other("Solana".to_string()),
            crate::types::EventType::Swap,
            program_id,
            input.instruction_index.clone(),
            input.received_time.timestamp_millis(),
        );

        Ok(SolanaEvent::new(metadata, event_data))
    }

    /// Extract program ID from instruction
    fn extract_program_id(
        &self,
        accounts: &[Pubkey],
        instruction: &solana_message::compiled_instruction::CompiledInstruction,
    ) -> EventResult<Pubkey> {
        accounts
            .get(instruction.program_id_index as usize)
            .copied()
            .ok_or_else(|| {
                EventError::parse_error(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid program ID index",
                    ),
                    "Failed to extract program ID from instruction",
                )
            })
    }
}

#[async_trait]
impl EventParser for SolanaEventParser {
    type Input = SolanaTransactionInput;

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let solana_events = self.parse_instruction(input).await?;

        Ok(solana_events
            .into_iter()
            .map(|event| Box::new(event) as Box<dyn Event>)
            .collect())
    }

    fn can_parse(&self, input: &Self::Input) -> bool {
        // Check if we have a supported program ID
        if let Some(program_id) = input
            .accounts
            .get(input.instruction.program_id_index as usize)
        {
            self.supports_program(program_id)
        } else {
            false
        }
    }

    fn info(&self) -> ParserInfo {
        self.info.clone()
    }
}

/// Inner instruction parser that implements the riglr-events-core EventParser trait
pub struct SolanaInnerInstructionParser {
    /// Inner Solana parser
    solana_parser: Arc<SolanaEventParser>,
}

impl SolanaInnerInstructionParser {
    /// Create new inner instruction parser
    pub fn new(solana_parser: Arc<SolanaEventParser>) -> Self {
        Self { solana_parser }
    }
}

#[async_trait]
impl EventParser for SolanaInnerInstructionParser {
    type Input = SolanaInnerInstructionInput;

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let solana_events = self.solana_parser.parse_inner_instruction(input).await?;

        Ok(solana_events
            .into_iter()
            .map(|event| Box::new(event) as Box<dyn Event>)
            .collect())
    }

    fn can_parse(&self, _input: &Self::Input) -> bool {
        // Inner instructions are generally parseable if we have the data
        // More sophisticated logic could be added here
        true
    }

    fn info(&self) -> ParserInfo {
        ParserInfo::new(
            "solana-inner-instruction-parser".to_string(),
            "1.0.0".to_string(),
        )
        .with_kind(riglr_events_core::types::EventKind::Transaction)
        .with_kind(riglr_events_core::types::EventKind::Swap)
        .with_kind(riglr_events_core::types::EventKind::Liquidity)
        .with_kind(riglr_events_core::types::EventKind::Transfer)
        .with_format("solana-inner-instruction".to_string())
    }
}

/// Utility functions for creating parser inputs
impl SolanaTransactionInput {
    /// Create a new Solana transaction input
    pub fn new(
        instruction: solana_message::compiled_instruction::CompiledInstruction,
        accounts: Vec<Pubkey>,
        signature: String,
        slot: u64,
        block_time: Option<i64>,
        instruction_index: usize,
    ) -> Self {
        Self {
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            received_time: chrono::Utc::now(),
            instruction_index,
        }
    }

    /// Set the received time
    pub fn with_received_time(mut self, received_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.received_time = received_time;
        self
    }
}

impl SolanaInnerInstructionInput {
    /// Create a new Solana inner instruction input
    pub fn new(
        inner_instruction: UiCompiledInstruction,
        signature: String,
        slot: u64,
        block_time: Option<i64>,
        instruction_index: String,
    ) -> Self {
        Self {
            inner_instruction,
            signature,
            slot,
            block_time,
            received_time: chrono::Utc::now(),
            instruction_index,
        }
    }

    /// Set the received time
    pub fn with_received_time(mut self, received_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.received_time = received_time;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_message::compiled_instruction::CompiledInstruction;

    #[tokio::test]
    async fn test_solana_parser_creation() {
        let parser = SolanaEventParser::default();

        assert_eq!(parser.info.name, "solana-event-parser");
        assert_eq!(parser.info.version, "1.0.0");
        assert!(!parser.info.supported_kinds.is_empty());
        assert!(parser
            .info
            .supported_formats
            .contains(&"solana-instruction".to_string()));
    }

    #[tokio::test]
    async fn test_solana_transaction_input() {
        let accounts = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: vec![1, 2, 3, 4],
        };

        let input = SolanaTransactionInput::new(
            instruction.clone(),
            accounts.clone(),
            "test-signature".to_string(),
            12345,
            Some(1234567890),
            0,
        );

        assert_eq!(input.signature, "test-signature");
        assert_eq!(input.slot, 12345);
        assert_eq!(input.block_time, Some(1234567890));
        assert_eq!(input.instruction_index, 0);
        assert_eq!(input.accounts, accounts);
        assert_eq!(input.instruction.data, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_input() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: "AQIDBA==".to_string(), // base58 encoded [1,2,3,4]
            stack_height: Some(1),
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction.clone(),
            "test-signature".to_string(),
            67890,
            Some(1234567890),
            "inner-0".to_string(),
        );

        assert_eq!(input.signature, "test-signature");
        assert_eq!(input.slot, 67890);
        assert_eq!(input.block_time, Some(1234567890));
        assert_eq!(input.instruction_index, "inner-0");
        assert_eq!(input.inner_instruction.data, "AQIDBA==");
    }

    #[tokio::test]
    async fn test_parser_can_parse() {
        let parser = SolanaEventParser::default();

        // Test with a random program ID (should return false since we don't have parsers set up)
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            None,
            0,
        );

        // This should be false because we don't have any parsers configured for random program IDs
        assert!(!parser.can_parse(&input));
    }

    #[tokio::test]
    async fn test_extract_program_id() {
        let parser = SolanaEventParser::default();
        let program_id = Pubkey::new_unique();
        let accounts = vec![program_id, Pubkey::new_unique()];

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: vec![],
        };

        let result = parser.extract_program_id(&accounts, &instruction);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), program_id);
    }

    #[tokio::test]
    async fn test_extract_program_id_invalid_index() {
        let parser = SolanaEventParser::default();
        let accounts = vec![Pubkey::new_unique()];

        let instruction = CompiledInstruction {
            program_id_index: 5, // Out of bounds
            accounts: vec![0],
            data: vec![],
        };

        let result = parser.extract_program_id(&accounts, &instruction);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_default_implementation() {
        let parser = SolanaEventParser::default();
        assert_eq!(parser.info.name, "solana-event-parser");
        assert_eq!(parser.info.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_with_legacy_parser() {
        let legacy_parser = EventParserRegistry::default();
        let parser = SolanaEventParser::with_legacy_parser(legacy_parser);
        assert_eq!(parser.info.name, "solana-event-parser");
        assert_eq!(parser.info.version, "1.0.0");
        assert!(!parser.info.supported_kinds.is_empty());
    }

    #[tokio::test]
    async fn test_add_protocol_parser() {
        let mut parser = SolanaEventParser::default();
        let mock_parser = Arc::new(MockEventParser {});
        // This method currently does nothing but we test it doesn't panic
        parser.add_protocol_parser::<MockEventParser>(Protocol::Jupiter, mock_parser);
    }

    #[tokio::test]
    async fn test_supports_program_true() {
        let parser = SolanaEventParser::default();
        let supported_programs = parser.supported_programs.clone();

        if let Some(program_id) = supported_programs.first() {
            assert!(parser.supports_program(program_id));
        }
    }

    #[tokio::test]
    async fn test_supports_program_false() {
        let parser = SolanaEventParser::default();
        let unsupported_program = Pubkey::new_unique();
        assert!(!parser.supports_program(&unsupported_program));
    }

    #[tokio::test]
    async fn test_can_parse_with_invalid_program_id_index() {
        let parser = SolanaEventParser::default();
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 10, // Out of bounds
            accounts: vec![0],
            data: vec![],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            None,
            0,
        );

        assert!(!parser.can_parse(&input));
    }

    #[tokio::test]
    async fn test_solana_transaction_input_with_received_time() {
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![1, 2, 3],
        };

        let custom_time = chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            0,
        )
        .with_received_time(custom_time);

        assert_eq!(input.received_time, custom_time);
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_input_with_received_time() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: "AQIDBA==".to_string(),
            stack_height: Some(1),
        };

        let custom_time = chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            "inner-0".to_string(),
        )
        .with_received_time(custom_time);

        assert_eq!(input.received_time, custom_time);
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_parser_new() {
        let solana_parser = Arc::new(SolanaEventParser::default());
        let inner_parser = SolanaInnerInstructionParser::new(solana_parser);

        let info = inner_parser.info();
        assert_eq!(info.name, "solana-inner-instruction-parser");
        assert_eq!(info.version, "1.0.0");
        assert!(info
            .supported_kinds
            .contains(&riglr_events_core::types::EventKind::Transaction));
        assert!(info
            .supported_formats
            .contains(&"solana-inner-instruction".to_string()));
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_parser_can_parse() {
        let solana_parser = Arc::new(SolanaEventParser::default());
        let inner_parser = SolanaInnerInstructionParser::new(solana_parser);

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: None,
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            None,
            "inner-0".to_string(),
        );

        // Should always return true for inner instructions
        assert!(inner_parser.can_parse(&input));
    }

    #[tokio::test]
    async fn test_extract_program_id_with_empty_accounts() {
        let parser = SolanaEventParser::default();
        let accounts = vec![];

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };

        let result = parser.extract_program_id(&accounts, &instruction);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_solana_transaction_input_with_none_block_time() {
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            None, // None block time
            0,
        );

        assert_eq!(input.block_time, None);
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_input_with_none_block_time() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: None,
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            None, // None block time
            "inner-0".to_string(),
        );

        assert_eq!(input.block_time, None);
    }

    #[tokio::test]
    async fn test_ui_compiled_instruction_with_none_stack_height() {
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: None, // None stack height
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction.clone(),
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            "inner-0".to_string(),
        );

        assert_eq!(input.inner_instruction.stack_height, None);
    }

    #[tokio::test]
    async fn test_parse_instruction_success() {
        let parser = SolanaEventParser::default();
        let accounts = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: vec![1, 2, 3, 4],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-signature".to_string(),
            12345,
            Some(1234567890),
            0,
        );

        // This should succeed even with no events returned from legacy parser
        let result = parser.parse_instruction(input).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        // Should be empty since we don't have actual parsers configured
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_parse_inner_instruction_success() {
        let parser = SolanaEventParser::default();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: "AQIDBA==".to_string(),
            stack_height: Some(1),
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-signature".to_string(),
            67890,
            Some(1234567890),
            "inner-0".to_string(),
        );

        let result = parser.parse_inner_instruction(input).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        // Should be empty since we don't have actual parsers configured
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_event_parser_trait_parse() {
        let parser = SolanaEventParser::default();
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![1, 2, 3],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            0,
        );

        let result = parser.parse(input).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events.is_empty()); // No events since no parsers configured
    }

    #[tokio::test]
    async fn test_event_parser_trait_info() {
        let parser = SolanaEventParser::default();
        let info = parser.info();

        assert_eq!(info.name, "solana-event-parser");
        assert_eq!(info.version, "1.0.0");
        assert!(info
            .supported_kinds
            .contains(&riglr_events_core::types::EventKind::Transaction));
        assert!(info
            .supported_kinds
            .contains(&riglr_events_core::types::EventKind::Swap));
        assert!(info
            .supported_kinds
            .contains(&riglr_events_core::types::EventKind::Liquidity));
        assert!(info
            .supported_kinds
            .contains(&riglr_events_core::types::EventKind::Transfer));
        assert!(info
            .supported_formats
            .contains(&"solana-instruction".to_string()));
        assert!(info
            .supported_formats
            .contains(&"solana-inner-instruction".to_string()));
    }

    #[tokio::test]
    async fn test_inner_instruction_parser_parse() {
        let solana_parser = Arc::new(SolanaEventParser::default());
        let inner_parser = SolanaInnerInstructionParser::new(solana_parser);

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: Some(2),
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            "inner-0".to_string(),
        );

        let result = inner_parser.parse(input).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events.is_empty()); // No events since no parsers configured
    }

    #[tokio::test]
    async fn test_extract_program_id_edge_case_max_index() {
        let parser = SolanaEventParser::default();
        let program_id = Pubkey::new_unique();
        let accounts = vec![program_id];

        let instruction = CompiledInstruction {
            program_id_index: u8::MAX, // Maximum possible index
            accounts: vec![0],
            data: vec![],
        };

        let result = parser.extract_program_id(&accounts, &instruction);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_solana_transaction_input_edge_cases() {
        // Test with maximum values
        let accounts = vec![Pubkey::new_unique(); 255];
        let instruction = CompiledInstruction {
            program_id_index: 254,
            accounts: (0..255).collect(),
            data: vec![0u8; 1024],
        };

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "a".repeat(88), // Maximum signature length
            u64::MAX,
            Some(i64::MAX),
            usize::MAX,
        );

        assert_eq!(input.slot, u64::MAX);
        assert_eq!(input.block_time, Some(i64::MAX));
        assert_eq!(input.instruction_index, usize::MAX);
        assert_eq!(input.signature.len(), 88);
        assert_eq!(input.accounts.len(), 255);
        assert_eq!(input.instruction.data.len(), 1024);
    }

    #[tokio::test]
    async fn test_solana_inner_instruction_input_edge_cases() {
        // Test with various edge case values
        let inner_instruction = UiCompiledInstruction {
            program_id_index: u8::MAX,
            accounts: (0..255).collect(),
            data: "x".repeat(1000), // Large data string
            stack_height: Some(u32::MAX),
        };

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-signature-with-long-name".to_string(),
            u64::MAX,
            Some(i64::MIN), // Minimum block time
            "inner-instruction-with-very-long-index-name".to_string(),
        );

        assert_eq!(input.slot, u64::MAX);
        assert_eq!(input.block_time, Some(i64::MIN));
        assert_eq!(input.inner_instruction.stack_height, Some(u32::MAX));
        assert_eq!(input.inner_instruction.program_id_index, u8::MAX);
        assert_eq!(input.inner_instruction.data.len(), 1000);
    }

    #[tokio::test]
    async fn test_debug_implementations() {
        // Test Debug trait implementations
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![1, 2, 3],
        };

        let tx_input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            0,
        );

        let debug_str = format!("{:?}", tx_input);
        assert!(debug_str.contains("SolanaTransactionInput"));
        assert!(debug_str.contains("test-sig"));
        assert!(debug_str.contains("12345"));

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: None,
        };

        let inner_input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            None,
            "inner-0".to_string(),
        );

        let debug_str = format!("{:?}", inner_input);
        assert!(debug_str.contains("SolanaInnerInstructionInput"));
        assert!(debug_str.contains("inner-0"));
    }

    #[tokio::test]
    async fn test_clone_implementations() {
        // Test Clone trait implementations
        let accounts = vec![Pubkey::new_unique()];
        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: vec![1, 2, 3],
        };

        let tx_input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            Some(1234567890),
            0,
        );

        let cloned_input = tx_input.clone();
        assert_eq!(tx_input.signature, cloned_input.signature);
        assert_eq!(tx_input.slot, cloned_input.slot);
        assert_eq!(tx_input.block_time, cloned_input.block_time);

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: "test".to_string(),
            stack_height: None,
        };

        let inner_input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            None,
            "inner-0".to_string(),
        );

        let cloned_inner = inner_input.clone();
        assert_eq!(inner_input.signature, cloned_inner.signature);
        assert_eq!(
            inner_input.instruction_index,
            cloned_inner.instruction_index
        );
    }

    // Mock event parser for testing
    struct MockEventParser;

    impl LegacyEventParser for MockEventParser {
        fn inner_instruction_configs(
            &self,
        ) -> std::collections::HashMap<
            &'static str,
            Vec<crate::events::core::traits::GenericEventParseConfig>,
        > {
            std::collections::HashMap::new()
        }

        fn instruction_configs(
            &self,
        ) -> std::collections::HashMap<
            Vec<u8>,
            Vec<crate::events::core::traits::GenericEventParseConfig>,
        > {
            std::collections::HashMap::new()
        }

        fn parse_events_from_inner_instruction(
            &self,
            _params: &crate::events::factory::InnerInstructionParseParams,
        ) -> Vec<Box<dyn riglr_events_core::Event>> {
            vec![]
        }

        fn parse_events_from_instruction(
            &self,
            _params: &crate::events::factory::InstructionParseParams,
        ) -> Vec<Box<dyn riglr_events_core::Event>> {
            vec![]
        }

        fn should_handle(&self, _program_id: &solana_sdk::pubkey::Pubkey) -> bool {
            false
        }

        fn supported_program_ids(&self) -> Vec<Pubkey> {
            vec![Pubkey::new_unique()]
        }
    }
}
