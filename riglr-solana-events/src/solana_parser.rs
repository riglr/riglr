//! Solana-specific event parser implementations using riglr-events-core.
//!
//! This module provides event parsers that implement the riglr-events-core EventParser trait
//! while leveraging the existing Solana parsing logic.

use std::sync::Arc;
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiCompiledInstruction;

use riglr_events_core::prelude::*;
use crate::events::core::traits::{EventParser as LegacyEventParser};
use crate::events::factory::{EventParserRegistry, Protocol, InstructionParseParams, InnerInstructionParseParams};
use crate::solana_events::SolanaEvent;

/// Input type for Solana transaction parsing
#[derive(Debug, Clone)]
pub struct SolanaTransactionInput {
    /// Solana instruction data
    pub instruction: solana_sdk::instruction::CompiledInstruction,
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

impl SolanaEventParser {
    /// Create a new Solana event parser
    pub fn new() -> Self {
        let legacy_parser = EventParserRegistry::new();
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
    pub fn add_protocol_parser<P>(&mut self, _protocol: Protocol, _parser: Arc<dyn LegacyEventParser>)
    where
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
    pub async fn parse_instruction(&self, input: SolanaTransactionInput) -> EventResult<Vec<SolanaEvent>> {
        let legacy_events = self.legacy_parser.parse_events_from_instruction(
            InstructionParseParams {
                instruction: &input.instruction,
                accounts: &input.accounts,
                signature: &input.signature,
                slot: input.slot,
                block_time: input.block_time,
                program_received_time_ms: input.received_time.timestamp_millis(),
                index: input.instruction_index.to_string(),
            }
        );

        let solana_events = legacy_events
            .into_iter()
            .map(|event| self.convert_legacy_event(event, &input))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(solana_events)
    }

    /// Parse events from a Solana inner instruction
    pub async fn parse_inner_instruction(&self, input: SolanaInnerInstructionInput) -> EventResult<Vec<SolanaEvent>> {
        let legacy_events = self.legacy_parser.parse_events_from_inner_instruction(
            InnerInstructionParseParams {
                inner_instruction: &input.inner_instruction,
                signature: &input.signature,
                slot: input.slot,
                block_time: input.block_time,
                program_received_time_ms: input.received_time.timestamp_millis(),
                index: input.instruction_index.clone(),
            }
        );

        let solana_events = legacy_events
            .into_iter()
            .map(|event| self.convert_legacy_inner_event(event, &input))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(solana_events)
    }

    /// Convert a legacy Event to a SolanaEvent
    fn convert_legacy_event(&self, event: Box<dyn riglr_events_core::Event>, input: &SolanaTransactionInput) -> EventResult<SolanaEvent> {
        // Extract data using the event's as_any method for downcasting
        let event_data = serde_json::json!({
            "kind": event.kind().to_string(),
            "signature": input.signature.clone(),
            "slot": input.slot,
            "timestamp": input.received_time.timestamp_millis(),
        });

        // Create metadata for the new event system
        let legacy_metadata = crate::types::EventMetadata::new(
            event.id().to_string(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            input.block_time.unwrap_or(0) * 1000,
            crate::types::ProtocolType::Other("Solana".to_string()),
            crate::types::EventType::Swap,
            self.extract_program_id(&input.accounts, &input.instruction)?,
            input.instruction_index.to_string(),
            input.received_time.timestamp_millis(),
        );

        Ok(SolanaEvent::new(legacy_metadata, event_data))
    }

    /// Convert a legacy Event from inner instruction to a SolanaEvent
    fn convert_legacy_inner_event(&self, event: Box<dyn riglr_events_core::Event>, input: &SolanaInnerInstructionInput) -> EventResult<SolanaEvent> {
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

        let legacy_metadata = crate::types::EventMetadata::new(
            event.id().to_string(),
            input.signature.clone(),
            input.slot,
            input.block_time.unwrap_or(0),
            input.block_time.unwrap_or(0) * 1000,
            crate::types::ProtocolType::Other("Solana".to_string()),
            crate::types::EventType::Swap,
            program_id,
            input.instruction_index.clone(),
            input.received_time.timestamp_millis(),
        );

        Ok(SolanaEvent::new(legacy_metadata, event_data))
    }

    /// Extract program ID from instruction
    fn extract_program_id(&self, accounts: &[Pubkey], instruction: &solana_sdk::instruction::CompiledInstruction) -> EventResult<Pubkey> {
        accounts
            .get(instruction.program_id_index as usize)
            .copied()
            .ok_or_else(|| EventError::parse_error(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid program ID index"),
                "Failed to extract program ID from instruction"
            ))
    }
}

impl Default for SolanaEventParser {
    fn default() -> Self {
        Self::new()
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
        if let Some(program_id) = input.accounts.get(input.instruction.program_id_index as usize) {
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
        ParserInfo::new("solana-inner-instruction-parser".to_string(), "1.0.0".to_string())
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
        instruction: solana_sdk::instruction::CompiledInstruction,
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
    use solana_sdk::instruction::CompiledInstruction;

    #[tokio::test]
    async fn test_solana_parser_creation() {
        let parser = SolanaEventParser::new();
        
        assert_eq!(parser.info.name, "solana-event-parser");
        assert_eq!(parser.info.version, "1.0.0");
        assert!(!parser.info.supported_kinds.is_empty());
        assert!(parser.info.supported_formats.contains(&"solana-instruction".to_string()));
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
        let parser = SolanaEventParser::new();
        
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
        let parser = SolanaEventParser::new();
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
        let parser = SolanaEventParser::new();
        let accounts = vec![Pubkey::new_unique()];
        
        let instruction = CompiledInstruction {
            program_id_index: 5, // Out of bounds
            accounts: vec![0],
            data: vec![],
        };

        let result = parser.extract_program_id(&accounts, &instruction);
        assert!(result.is_err());
    }
}