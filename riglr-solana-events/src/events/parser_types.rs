//! Solana-specific parser types and configurations
//!
//! This module contains types used by Solana event parsers that don't conflict
//! with the core EventParser trait from riglr_events_core.

use crate::error::ParseResult;
use crate::metadata_helpers;
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};
use riglr_events_core::Event;
use std::fmt::Debug;

type EventMetadata = SolanaEventMetadata;

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

    /// Get inner instruction parsing configurations
    pub fn inner_instruction_configs(
        &self,
    ) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    /// Get instruction parsing configurations
    pub fn instruction_configs(
        &self,
    ) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    /// Parse event data from inner instruction
    pub fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let core_metadata = metadata_helpers::create_core_metadata(
                        format!("{}_{}", params.signature, params.index),
                        riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                        "solana".to_string(),
                        params.block_time,
                    );

                    let metadata = SolanaEventMetadata::new(
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

    /// Parse event data from instruction
    pub fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        if let Some(configs) = self.instruction_configs.get(&params.instruction.data) {
            for config in configs {
                let core_metadata = metadata_helpers::create_core_metadata(
                    format!("{}_{}", params.signature, params.index),
                    riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                    "solana".to_string(),
                    params.block_time,
                );

                let metadata = SolanaEventMetadata::new(
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

    /// Check if this program ID should be handled
    pub fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    /// Get supported program ID list
    pub fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.program_ids.clone()
    }
}

/// Protocol-specific parser trait for Solana events
/// This trait is used internally by protocol parsers and doesn't conflict with
/// the core EventParser trait from riglr_events_core.
pub trait ProtocolParser: Send + Sync {
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
