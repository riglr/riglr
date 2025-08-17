//! Solana-specific metadata wrapper for events
//!
//! This module provides a Solana-specific metadata type that wraps the core EventMetadata
//! and adds support for BorshDeserialize and other Solana-specific requirements.

use crate::types::{EventType, ProtocolType};
use borsh::{BorshDeserialize, BorshSerialize};
use riglr_events_core::{EventKind, EventMetadata};
use serde::{Deserialize, Serialize};

/// Solana-specific event metadata that wraps core EventMetadata
/// and provides additional fields and trait implementations
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SolanaEventMetadata {
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
    /// Event type
    pub event_type: EventType,
    /// Protocol type
    pub protocol_type: ProtocolType,
    /// Instruction index
    pub index: String,
    /// Program received time in milliseconds
    pub program_received_time_ms: i64,
    /// The underlying core metadata (excluded from borsh serialization)
    #[borsh(skip)]
    #[serde(skip)]
    pub core: EventMetadata,
}

impl SolanaEventMetadata {
    /// Create new Solana event metadata
    pub fn new(
        signature: String,
        slot: u64,
        event_type: EventType,
        protocol_type: ProtocolType,
        index: String,
        program_received_time_ms: i64,
        core: EventMetadata,
    ) -> Self {
        Self {
            signature,
            slot,
            event_type,
            protocol_type,
            index,
            program_received_time_ms,
            core,
        }
    }

    /// Set the event ID
    pub fn set_id(&mut self, id: String) {
        self.core.id = id;
    }

    /// Get the event ID
    pub fn id(&self) -> &str {
        &self.core.id
    }

    /// Get the event kind from core metadata
    pub fn kind(&self) -> &EventKind {
        &self.core.kind
    }

    /// Convert event type to event kind
    pub fn event_kind(&self) -> EventKind {
        match self.event_type {
            EventType::Swap => EventKind::Swap,
            EventType::Transfer => EventKind::Transfer,
            EventType::Liquidate => EventKind::Custom("liquidation".to_string()),
            EventType::Deposit => EventKind::Custom("deposit".to_string()),
            EventType::Withdraw => EventKind::Custom("withdraw".to_string()),
            EventType::Borrow => EventKind::Custom("borrow".to_string()),
            EventType::Repay => EventKind::Custom("repay".to_string()),
            EventType::CreatePool => EventKind::Custom("create_pool".to_string()),
            EventType::AddLiquidity => EventKind::Custom("add_liquidity".to_string()),
            EventType::RemoveLiquidity => EventKind::Custom("remove_liquidity".to_string()),
            _ => EventKind::Custom("unknown".to_string()),
        }
    }

    /// Get a reference to the core EventMetadata
    pub fn core(&self) -> &EventMetadata {
        &self.core
    }

    /// Get a mutable reference to the core EventMetadata
    pub fn core_mut(&mut self) -> &mut EventMetadata {
        &mut self.core
    }
}

impl Default for SolanaEventMetadata {
    fn default() -> Self {
        use crate::metadata_helpers::create_solana_metadata;

        let core = create_solana_metadata(
            String::new(),
            EventKind::Transaction,
            "solana".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        Self {
            signature: String::new(),
            slot: 0,
            event_type: EventType::default(),
            protocol_type: ProtocolType::default(),
            index: String::new(),
            program_received_time_ms: 0,
            core,
        }
    }
}

/// Create a SolanaEventMetadata from components
pub fn create_metadata(
    id: String,
    signature: String,
    slot: u64,
    block_time: Option<i64>,
    received_time: i64,
    index: String,
    event_type: EventType,
    protocol_type: ProtocolType,
) -> SolanaEventMetadata {
    use crate::metadata_helpers::create_solana_metadata;

    let core = create_solana_metadata(
        id.clone(),
        EventKind::Transaction,
        "solana".to_string(),
        slot,
        Some(signature.clone()),
        None,
        None,
        block_time,
        protocol_type.clone(),
        event_type.clone(),
    );

    SolanaEventMetadata {
        signature,
        slot,
        event_type,
        protocol_type,
        index,
        program_received_time_ms: received_time,
        core,
    }
}
