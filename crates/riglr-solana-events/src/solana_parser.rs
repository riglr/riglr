//! Solana-specific event parser implementations using riglr-events-core.
//!
//! This module provides event parsers that implement the riglr-events-core EventParser trait
//! while leveraging the existing Solana parsing logic.

extern crate alloc;

use alloc::sync::Arc;
use async_trait::async_trait;
use riglr_events_core::types::EventKind;
use solana_message::compiled_instruction::CompiledInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiCompiledInstruction;
use std::io::{Error as IoError, ErrorKind};

// Legacy EventParser trait has been removed - using riglr_events_core::traits::EventParser
use crate::events::factory::{
    EventParserRegistry, InnerInstructionParseParams, InstructionParseParams,
};
use crate::solana_events::SolanaEvent;
use riglr_events_core::prelude::*;

/// Input type for Solana transaction parsing
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SolanaTransactionInput {
    /// Account keys from the transaction
    pub accounts: Vec<Pubkey>,
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Solana instruction data
    pub instruction: CompiledInstruction,
    /// Instruction index for identification
    pub instruction_index: usize,
    /// When the event was received by the parser
    pub received_time: chrono::DateTime<chrono::Utc>,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
}

/// Input type for Solana inner instruction parsing
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SolanaInnerInstructionInput {
    /// Block time (optional)
    pub block_time: Option<i64>,
    /// Inner instruction data
    pub inner_instruction: UiCompiledInstruction,
    /// Instruction index for identification
    pub instruction_index: String,
    /// When the event was received by the parser
    pub received_time: chrono::DateTime<chrono::Utc>,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
}

/// Solana event parser that bridges between legacy and new parsers
#[derive(Debug)]
pub struct SolanaEventParser {
    /// Parser information
    info: ParserInfo,
    /// Legacy multi-parser for actual parsing logic
    legacy_parser: EventParserRegistry,
    /// Supported program IDs
    supported_programs: Vec<Pubkey>,
}

impl Default for SolanaEventParser {
    #[inline]
    fn default() -> Self {
        let legacy_parser = EventParserRegistry::default();
        let supported_programs = legacy_parser.supported_program_ids();
        Self {
            info: ParserInfo::new("solana-event-parser".to_owned(), "1.0.0".to_owned())
                .with_kind(EventKind::Transaction)
                .with_kind(EventKind::Swap)
                .with_kind(EventKind::Liquidity)
                .with_kind(EventKind::Transfer)
                .with_format("solana-instruction".to_owned())
                .with_format("solana-inner-instruction".to_owned()),
            legacy_parser,
            supported_programs,
        }
    }
}

impl SolanaEventParser {
    /// Convert a legacy Event to a `SolanaEvent`
    fn convert_legacy_event(
        event: &dyn riglr_events_core::Event,
        input: &SolanaTransactionInput,
    ) -> EventResult<SolanaEvent> {
        use crate::types::{metadata_helpers::create_solana_metadata, EventType, ProtocolType};

        // Extract data using the event's as_any method for downcasting
        let event_data = serde_json::json!({
            "kind": event.kind().to_string(),
            "signature": input.signature.clone(),
            "slot": input.slot,
            "timestamp": input.received_time.timestamp_millis(),
        });

        // Create metadata for the new event system
        let metadata = create_solana_metadata(
            event.id().to_owned(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            ProtocolType::Other("Solana".to_owned()),
            EventType::Swap,
            Self::extract_program_id(&input.accounts, &input.instruction)?,
            input.instruction_index.to_string(),
            input.received_time.timestamp_millis(),
        );

        Ok(SolanaEvent::new(metadata, event_data))
    }

    /// Convert a legacy Event from inner instruction to a `SolanaEvent`
    fn convert_legacy_inner_event(
        event: &dyn riglr_events_core::Event,
        input: &SolanaInnerInstructionInput,
    ) -> SolanaEvent {
        use crate::types::{metadata_helpers::create_solana_metadata, EventType, ProtocolType};

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

        let metadata = create_solana_metadata(
            event.id().to_owned(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            ProtocolType::Other("Solana".to_owned()),
            EventType::Swap,
            program_id,
            input.instruction_index.clone(),
            input.received_time.timestamp_millis(),
        );

        SolanaEvent::new(metadata, event_data)
    }

    /// Extract program ID from instruction
    fn extract_program_id(
        accounts: &[Pubkey],
        instruction: &CompiledInstruction,
    ) -> EventResult<Pubkey> {
        accounts
            .get(instruction.program_id_index as usize)
            .copied()
            .ok_or_else(|| {
                EventError::parse_error(
                    IoError::new(ErrorKind::InvalidData, "Invalid program ID index"),
                    "Failed to extract program ID from instruction",
                )
            })
    }

    /// Create a new Solana event parser
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let legacy_parser = EventParserRegistry::default();
        let supported_programs = legacy_parser.supported_program_ids();
        Self {
            info: ParserInfo::new("solana-event-parser".to_owned(), "1.0.0".to_owned())
                .with_kind(EventKind::Transaction)
                .with_kind(EventKind::Swap)
                .with_kind(EventKind::Liquidity)
                .with_kind(EventKind::Transfer)
                .with_format("solana-instruction".to_owned())
                .with_format("solana-inner-instruction".to_owned()),
            legacy_parser,
            supported_programs,
        }
    }

    /// Parse events from a Solana inner instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the inner instruction cannot be parsed or if there are issues
    /// with the legacy event conversion process.
    #[inline]
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
            .iter()
            .map(|event| Self::convert_legacy_inner_event(event.as_ref(), &input))
            .collect::<Vec<_>>();

        Ok(solana_events)
    }

    /// Parse events from a Solana instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction cannot be parsed or if there are issues
    /// with the legacy event conversion process.
    #[inline]
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
            .iter()
            .map(|event| Self::convert_legacy_event(event.as_ref(), &input))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(solana_events)
    }

    /// Check if a program ID is supported
    #[must_use]
    #[inline]
    pub fn supports_program(&self, program_id: &Pubkey) -> bool {
        self.supported_programs.contains(program_id)
    }

    /// Create with specific legacy parser
    #[must_use]
    #[inline]
    pub fn with_legacy_parser(legacy_parser: EventParserRegistry) -> Self {
        let supported_programs = legacy_parser.supported_program_ids();

        Self {
            info: ParserInfo::new("solana-event-parser".to_owned(), "1.0.0".to_owned())
                .with_kind(EventKind::Transaction)
                .with_kind(EventKind::Swap)
                .with_kind(EventKind::Liquidity)
                .with_kind(EventKind::Transfer)
                .with_format("solana-instruction".to_owned())
                .with_format("solana-inner-instruction".to_owned()),
            legacy_parser,
            supported_programs,
        }
    }
}

#[async_trait]
impl EventParser for SolanaEventParser {
    type Input = SolanaTransactionInput;

    #[inline]
    fn can_parse(&self, input: &Self::Input) -> bool {
        // Check if we have a supported program ID
        let result = input
            .accounts
            .get(input.instruction.program_id_index as usize)
            .is_some_and(|program_id| self.supports_program(program_id));
        result
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }
    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let solana_events = self.parse_instruction(input).await?;

        return Ok(solana_events
            .into_iter()
            .map(|event| {
                let boxed: Box<dyn Event> = Box::new(event);
                boxed
            })
            .collect());
    }
}

/// Inner instruction parser that implements the riglr-events-core `EventParser` trait
#[derive(Debug)]
pub struct SolanaInnerInstructionParser {
    /// Parser information
    info: ParserInfo,
    /// Inner Solana parser
    solana_parser: Arc<SolanaEventParser>,
}

impl SolanaInnerInstructionParser {
    /// Create new inner instruction parser
    #[must_use]
    #[inline]
    pub fn new(solana_parser: Arc<SolanaEventParser>) -> Self {
        Self {
            info: ParserInfo::new(
                "solana-inner-instruction-parser".to_owned(),
                "1.0.0".to_owned(),
            )
            .with_kind(EventKind::Transaction)
            .with_kind(EventKind::Swap)
            .with_kind(EventKind::Liquidity)
            .with_kind(EventKind::Transfer)
            .with_format("solana-inner-instruction".to_owned()),
            solana_parser,
        }
    }
}

#[async_trait]
impl EventParser for SolanaInnerInstructionParser {
    type Input = SolanaInnerInstructionInput;

    #[inline]
    fn can_parse(&self, _input: &Self::Input) -> bool {
        // Inner instructions are generally parseable if we have the data
        // More sophisticated logic could be added here
        true
    }

    #[inline]
    fn info(&self) -> &ParserInfo {
        &self.info
    }
    #[inline]
    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let solana_events = self.solana_parser.parse_inner_instruction(input).await?;

        return Ok(solana_events
            .into_iter()
            .map(|event| {
                let boxed: Box<dyn Event> = Box::new(event);
                boxed
            })
            .collect());
    }
}

/// Utility functions for creating parser inputs
impl SolanaTransactionInput {
    /// Create a new Solana transaction input
    #[must_use]
    #[inline]
    pub fn new(
        instruction: CompiledInstruction,
        accounts: Vec<Pubkey>,
        signature: String,
        slot: u64,
        block_time: Option<i64>,
        instruction_index: usize,
    ) -> Self {
        Self {
            accounts,
            block_time,
            instruction,
            instruction_index,
            received_time: chrono::Utc::now(),
            signature,
            slot,
        }
    }

    /// Set the received time
    #[inline]
    #[must_use]
    pub const fn with_received_time(
        mut self,
        received_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.received_time = received_time;
        self
    }
}

impl SolanaInnerInstructionInput {
    /// Create a new Solana inner instruction input
    #[must_use]
    #[inline]
    pub fn new(
        inner_instruction: UiCompiledInstruction,
        signature: String,
        slot: u64,
        block_time: Option<i64>,
        instruction_index: String,
    ) -> Self {
        Self {
            block_time,
            inner_instruction,
            instruction_index,
            received_time: chrono::Utc::now(),
            signature,
            slot,
        }
    }

    /// Set the received time
    #[inline]
    #[must_use]
    pub const fn with_received_time(
        mut self,
        received_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.received_time = received_time;
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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
            instruction,
            accounts.clone(),
            "test-signature".to_string(),
            12345,
            Some(1_234_567_890),
            0,
        );

        assert_eq!(input.signature, "test-signature");
        assert_eq!(input.slot, 12345);
        assert_eq!(input.block_time, Some(1_234_567_890));
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
            inner_instruction,
            "test-signature".to_string(),
            67890,
            Some(1_234_567_890),
            "inner-0".to_string(),
        );

        assert_eq!(input.signature, "test-signature");
        assert_eq!(input.slot, 67890);
        assert_eq!(input.block_time, Some(1_234_567_890));
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
        let program_id = Pubkey::new_unique();
        let accounts = vec![program_id, Pubkey::new_unique()];

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1],
            data: vec![],
        };

        let result = SolanaEventParser::extract_program_id(&accounts, &instruction);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("Valid program ID extraction should succeed"),
            program_id
        );
    }

    #[tokio::test]
    async fn test_extract_program_id_invalid_index() {
        let accounts = vec![Pubkey::new_unique()];

        let instruction = CompiledInstruction {
            program_id_index: 5, // Out of bounds
            accounts: vec![0],
            data: vec![],
        };

        let result = SolanaEventParser::extract_program_id(&accounts, &instruction);
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

    // Test removed: add_protocol_parser method and MockEventParser struct were part of the legacy
    // EventParser trait which has been removed. Tests should use riglr_events_core::traits::EventParser implementations instead.

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
            .expect("Valid RFC3339 datetime string should parse successfully")
            .with_timezone(&chrono::Utc);

        let input = SolanaTransactionInput::new(
            instruction,
            accounts,
            "test-sig".to_string(),
            12345,
            Some(1_234_567_890),
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
            .expect("Valid RFC3339 datetime string should parse successfully")
            .with_timezone(&chrono::Utc);

        let input = SolanaInnerInstructionInput::new(
            inner_instruction,
            "test-sig".to_string(),
            12345,
            Some(1_234_567_890),
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
        assert!(info.supported_kinds.contains(&EventKind::Transaction));
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
        let accounts = vec![];

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![],
        };

        let result = SolanaEventParser::extract_program_id(&accounts, &instruction);
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
            inner_instruction,
            "test-sig".to_string(),
            12345,
            Some(1_234_567_890),
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
            Some(1_234_567_890),
            0,
        );

        // This should succeed even with no events returned from legacy parser
        let result = parser.parse_instruction(input).await;
        assert!(result.is_ok());
        let events = result.expect("Parsing instruction should succeed with empty parser registry");
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
            Some(1_234_567_890),
            "inner-0".to_string(),
        );

        let result = parser.parse_inner_instruction(input).await;
        assert!(result.is_ok());
        let events = result.expect("Parsing instruction should succeed with empty parser registry");
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
            Some(1_234_567_890),
            0,
        );

        let result = parser.parse(input).await;
        assert!(result.is_ok());
        let events = result.expect("Parsing instruction should succeed with empty parser registry");
        assert!(events.is_empty()); // No events since no parsers configured
    }

    #[tokio::test]
    async fn test_event_parser_trait_info() {
        let parser = SolanaEventParser::default();
        let info = parser.info();

        assert_eq!(info.name, "solana-event-parser");
        assert_eq!(info.version, "1.0.0");
        assert!(info.supported_kinds.contains(&EventKind::Transaction));
        assert!(info.supported_kinds.contains(&EventKind::Swap));
        assert!(info.supported_kinds.contains(&EventKind::Liquidity));
        assert!(info.supported_kinds.contains(&EventKind::Transfer));
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
            Some(1_234_567_890),
            "inner-0".to_string(),
        );

        let result = inner_parser.parse(input).await;
        assert!(result.is_ok());
        let events = result.expect("Parsing instruction should succeed with empty parser registry");
        assert!(events.is_empty()); // No events since no parsers configured
    }

    #[tokio::test]
    async fn test_extract_program_id_edge_case_max_index() {
        let program_id = Pubkey::new_unique();
        let accounts = vec![program_id];

        let instruction = CompiledInstruction {
            program_id_index: u8::MAX, // Maximum possible index
            accounts: vec![0],
            data: vec![],
        };

        let result = SolanaEventParser::extract_program_id(&accounts, &instruction);
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
            Some(1_234_567_890),
            0,
        );

        let debug_str = format!("{tx_input:?}");
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

        let debug_str = format!("{inner_input:?}");
        assert!(debug_str.contains("SolanaInnerInstructionInput"));
        assert!(debug_str.contains("inner-0"));

        // Test the parser structs themselves now have Debug implementations
        let parser = SolanaEventParser::default();
        let parser_debug = format!("{parser:?}");
        assert!(parser_debug.contains("SolanaEventParser"));

        let inner_parser = SolanaInnerInstructionParser::new(Arc::new(parser));
        let inner_parser_debug = format!("{inner_parser:?}");
        assert!(inner_parser_debug.contains("SolanaInnerInstructionParser"));
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
            Some(1_234_567_890),
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

    // MockEventParser removed: Used the legacy EventParser trait which has been removed.
    // Tests should use riglr_events_core::traits::EventParser implementations instead.
}
