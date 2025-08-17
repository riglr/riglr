//! Helper functions for working with Solana-specific EventMetadata
//!
//! This module provides utilities to properly create and access Solana event metadata
//! through the ChainData abstraction from riglr-events-core.

use crate::types::{EventType, ProtocolType};
use riglr_events_core::prelude::ChainData;
use riglr_events_core::{EventKind, EventMetadata};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;

/// Create a new EventMetadata for Solana events with all required fields
pub fn create_solana_metadata(
    id: String,
    kind: EventKind,
    source: String,
    slot: u64,
    signature: Option<String>,
    program_id: Option<Pubkey>,
    instruction_index: Option<usize>,
    block_time: Option<i64>,
    protocol_type: ProtocolType,
    event_type: EventType,
) -> EventMetadata {
    let protocol_data = json!({
        "protocol_type": protocol_type,
        "event_type": event_type,
    });

    EventMetadata::new(id, kind, source).with_chain_data(ChainData::Solana {
        slot,
        signature,
        program_id,
        instruction_index,
        block_time,
        protocol_data: Some(protocol_data),
    })
}

/// Get slot from Solana EventMetadata
pub fn get_slot(metadata: &EventMetadata) -> Option<u64> {
    match &metadata.chain_data {
        Some(ChainData::Solana { slot, .. }) => Some(*slot),
        _ => None,
    }
}

/// Get signature from Solana EventMetadata
pub fn get_signature(metadata: &EventMetadata) -> Option<&str> {
    match &metadata.chain_data {
        Some(ChainData::Solana { signature, .. }) => signature.as_deref(),
        _ => None,
    }
}

/// Get program_id from Solana EventMetadata
pub fn get_program_id(metadata: &EventMetadata) -> Option<&Pubkey> {
    match &metadata.chain_data {
        Some(ChainData::Solana { program_id, .. }) => program_id.as_ref(),
        _ => None,
    }
}

/// Get instruction_index from Solana EventMetadata
pub fn get_instruction_index(metadata: &EventMetadata) -> Option<usize> {
    match &metadata.chain_data {
        Some(ChainData::Solana {
            instruction_index, ..
        }) => *instruction_index,
        _ => None,
    }
}

/// Get block_time from Solana EventMetadata
pub fn get_block_time(metadata: &EventMetadata) -> Option<i64> {
    match &metadata.chain_data {
        Some(ChainData::Solana { block_time, .. }) => *block_time,
        _ => None,
    }
}

/// Get protocol_type from Solana EventMetadata
pub fn get_protocol_type(metadata: &EventMetadata) -> Option<ProtocolType> {
    match &metadata.chain_data {
        Some(ChainData::Solana { protocol_data, .. }) => protocol_data
            .as_ref()
            .and_then(|data| data.get("protocol_type"))
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        _ => None,
    }
}

/// Get event_type from Solana EventMetadata
pub fn get_event_type(metadata: &EventMetadata) -> Option<EventType> {
    match &metadata.chain_data {
        Some(ChainData::Solana { protocol_data, .. }) => protocol_data
            .as_ref()
            .and_then(|data| data.get("event_type"))
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        _ => None,
    }
}

/// Set protocol_type in Solana EventMetadata
pub fn set_protocol_type(metadata: &mut EventMetadata, protocol_type: ProtocolType) {
    if let Some(ChainData::Solana { protocol_data, .. }) = &mut metadata.chain_data {
        let data = protocol_data.get_or_insert_with(|| json!({}));
        if let Some(obj) = data.as_object_mut() {
            obj.insert("protocol_type".to_string(), json!(protocol_type));
        }
    }
}

/// Set event_type in Solana EventMetadata
pub fn set_event_type(metadata: &mut EventMetadata, event_type: EventType) {
    if let Some(ChainData::Solana { protocol_data, .. }) = &mut metadata.chain_data {
        let data = protocol_data.get_or_insert_with(|| json!({}));
        if let Some(obj) = data.as_object_mut() {
            obj.insert("event_type".to_string(), json!(event_type));
        }
    }
}

/// Helper struct that wraps EventMetadata for Solana-specific operations
/// This is primarily used in legacy code that expects direct field access
pub struct SolanaEventMetadata {
    pub inner: EventMetadata,
}

impl SolanaEventMetadata {
    /// Create a new SolanaEventMetadata
    pub fn new(
        id: String,
        kind: EventKind,
        source: String,
        slot: u64,
        signature: Option<String>,
        program_id: Option<Pubkey>,
        instruction_index: Option<usize>,
        block_time: Option<i64>,
        protocol_type: ProtocolType,
        event_type: EventType,
    ) -> Self {
        Self {
            inner: create_solana_metadata(
                id,
                kind,
                source,
                slot,
                signature,
                program_id,
                instruction_index,
                block_time,
                protocol_type,
                event_type,
            ),
        }
    }

    /// Get the slot
    pub fn slot(&self) -> u64 {
        get_slot(&self.inner).unwrap_or(0)
    }

    /// Get the signature
    pub fn signature(&self) -> Option<String> {
        get_signature(&self.inner).map(|s| s.to_string())
    }

    /// Get the protocol type
    pub fn protocol_type(&self) -> ProtocolType {
        get_protocol_type(&self.inner).unwrap_or_default()
    }

    /// Get the event type
    pub fn event_type(&self) -> EventType {
        get_event_type(&self.inner).unwrap_or_default()
    }

    /// Get the instruction index
    pub fn index(&self) -> Option<usize> {
        get_instruction_index(&self.inner)
    }

    /// Set the event type
    pub fn set_event_type(&mut self, event_type: EventType) {
        set_event_type(&mut self.inner, event_type);
    }

    /// Set the protocol type
    pub fn set_protocol_type(&mut self, protocol_type: ProtocolType) {
        set_protocol_type(&mut self.inner, protocol_type);
    }

    /// Convert to the inner EventMetadata
    pub fn into_inner(self) -> EventMetadata {
        self.inner
    }
}

impl From<SolanaEventMetadata> for EventMetadata {
    fn from(metadata: SolanaEventMetadata) -> Self {
        metadata.inner
    }
}

impl AsRef<EventMetadata> for SolanaEventMetadata {
    fn as_ref(&self) -> &EventMetadata {
        &self.inner
    }
}

impl Default for SolanaEventMetadata {
    fn default() -> Self {
        Self {
            inner: create_solana_metadata(
                "".to_string(),
                EventKind::Transaction,
                "solana".to_string(),
                0,
                None,
                None,
                None,
                None,
                ProtocolType::default(),
                EventType::default(),
            ),
        }
    }
}
