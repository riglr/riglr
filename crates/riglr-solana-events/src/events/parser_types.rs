//! Solana-specific parser types and configurations
//!
//! This module contains types used by Solana event parsers that don't conflict
//! with the core EventParser trait from riglr_events_core.

use crate::error::ParseResult;
use crate::events::factory::{InnerInstructionParseParams, InstructionParseParams};
use crate::metadata_helpers;
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType};
use core::fmt::Debug;
use riglr_events_core::Event;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Convenient type alias for Solana event metadata used in parser functions
type EventMetadata = SolanaEventMetadata;

/// Generic event parser configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GenericEventParseConfig {
    /// Type of events this configuration generates
    pub event_type: EventType,
    /// Discriminator string for inner instructions
    pub inner_instruction_discriminator: &'static str,
    /// Parser function for inner instructions
    pub inner_instruction_parser: InnerInstructionEventParser,
    /// Discriminator bytes for instructions
    pub instruction_discriminator: &'static [u8],
    /// Parser function for instructions
    pub instruction_parser: InstructionEventParser,
    /// Program ID this configuration applies to
    pub program_id: Pubkey,
    /// Protocol type for events generated from this configuration
    pub protocol_type: ProtocolType,
}

/// Inner instruction event parser
pub type InnerInstructionEventParser =
    for<'data> fn(data: &'data [u8], metadata: EventMetadata) -> ParseResult<Box<dyn Event>>;

/// Instruction event parser
pub type InstructionEventParser = for<'data> fn(
    data: &'data [u8],
    accounts: &'data [Pubkey],
    metadata: EventMetadata,
) -> ParseResult<Box<dyn Event>>;

/// Generic event parser base class
#[derive(Debug)]
#[non_exhaustive]
pub struct GenericEventParser {
    /// Configuration mapping for inner instruction parsing by discriminator
    pub inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    /// Configuration mapping for instruction parsing by discriminator bytes
    pub instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    /// List of program IDs this parser handles
    pub program_ids: Vec<Pubkey>,
}

impl GenericEventParser {
    /// Get inner instruction parsing configurations
    #[must_use]
    #[inline]
    pub fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    /// Get instruction parsing configurations
    #[must_use]
    #[inline]
    pub fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    /// Create new generic event parser
    #[inline]
    pub fn new(program_ids: Vec<Pubkey>, configs: Vec<GenericEventParseConfig>) -> Self {
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
            inner_instruction_configs,
            instruction_configs,
            program_ids,
        }
    }

    /// Parse event data from inner instruction
    #[must_use]
    #[inline]
    pub fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            // Allow hash iteration as the order doesn't matter for event parsing
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let core_metadata = metadata_helpers::create_core_metadata(
                        format!("{}_{}", params.signature, params.index),
                        riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                        "solana".to_owned(),
                        params.block_time,
                    );

                    let metadata = SolanaEventMetadata::new(
                        params.signature.to_owned(),
                        params.slot,
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                        params.index.clone(),
                        params.program_received_time_ms,
                        core_metadata,
                    );

                    if let Ok(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                    // Note: We continue processing other configs on error
                }
            }
        }

        events
    }

    /// Parse event data from instruction
    #[must_use]
    #[inline]
    pub fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        if let Some(configs) = self.instruction_configs.get(&params.instruction.data) {
            for config in configs {
                let core_metadata = metadata_helpers::create_core_metadata(
                    format!("{}_{}", params.signature, params.index),
                    riglr_events_core::EventKind::Custom(config.event_type.to_string()),
                    "solana".to_owned(),
                    params.block_time,
                );

                let metadata = SolanaEventMetadata::new(
                    params.signature.to_owned(),
                    params.slot,
                    config.event_type.clone(),
                    config.protocol_type.clone(),
                    params.index.clone(),
                    params.program_received_time_ms,
                    core_metadata,
                );

                if let Ok(event) =
                    (config.instruction_parser)(&params.instruction.data, params.accounts, metadata)
                {
                    events.push(event);
                }
                // Note: We continue processing other configs on error
            }
        }

        events
    }

    /// Check if this program ID should be handled
    #[must_use]
    #[inline]
    pub fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    /// Get supported program ID list
    #[must_use]
    #[inline]
    pub fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }
}

/// Protocol-specific parser trait for Solana events
/// This trait is used internally by protocol parsers and doesn't conflict with
/// the core `EventParser` trait from `riglr_events_core`.
pub trait ProtocolParser: Send + Sync {
    /// Get inner instruction parsing configurations
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>>;

    /// Get instruction parsing configurations
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>>;

    /// Parse event data from inner instruction
    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>>;

    /// Parse event data from instruction
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>>;

    /// Check if this program ID should be handled
    fn should_handle(&self, program_id: &Pubkey) -> bool;

    /// Get supported program ID list
    fn supported_program_ids(&self) -> Vec<Pubkey>;
}
