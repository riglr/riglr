//! Solana-specific metadata wrapper for events
//!
//! This module provides a Solana-specific metadata type that wraps the core EventMetadata
//! and adds support for BorshDeserialize and other Solana-specific requirements.

use crate::types::{EventType, ProtocolType};
use borsh::{BorshDeserialize, BorshSerialize};
use riglr_events_core::{EventKind, EventMetadata};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

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
        self.event_type.to_event_kind()
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

// Implement Deref to allow direct access to core EventMetadata fields
impl Deref for SolanaEventMetadata {
    type Target = EventMetadata;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for SolanaEventMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl Default for SolanaEventMetadata {
    fn default() -> Self {
        use crate::metadata_helpers::create_core_metadata;

        let core = create_core_metadata(
            String::default(),      // id
            EventKind::Transaction, // kind
            "solana".to_string(),   // source
            Some(0),                // block_time
        );

        Self {
            signature: String::default(),
            slot: 0,
            event_type: EventType::default(),
            protocol_type: ProtocolType::default(),
            index: String::default(),
            program_received_time_ms: 0,
            core,
        }
    }
}

/// Create a SolanaEventMetadata from components
#[allow(clippy::too_many_arguments)]
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
    use crate::metadata_helpers::create_core_metadata;

    let core = create_core_metadata(
        id,                     // id
        EventKind::Transaction, // kind
        "solana".to_string(),   // source
        block_time,             // block_time
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

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::EventKind;

    fn create_test_core_metadata() -> EventMetadata {
        use crate::metadata_helpers::create_core_metadata;
        use riglr_events_core::EventKind;

        create_core_metadata(
            "test-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
            Some(1234567890),
        )
    }

    #[test]
    fn test_new_when_valid_inputs_should_create_instance() {
        // Happy path
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "signature123".to_string(),
            12345,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1234567890,
            core.clone(),
        );

        assert_eq!(metadata.signature, "signature123");
        assert_eq!(metadata.slot, 12345);
        assert_eq!(metadata.event_type, EventType::Swap);
        assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
        assert_eq!(metadata.index, "0");
        assert_eq!(metadata.program_received_time_ms, 1234567890);
        assert_eq!(metadata.core, core);
    }

    #[test]
    fn test_new_when_empty_signature_should_create_instance() {
        // Edge case: empty signature
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "".to_string(),
            0,
            EventType::Unknown,
            ProtocolType::default(),
            "".to_string(),
            0,
            core,
        );

        assert_eq!(metadata.signature, "");
        assert_eq!(metadata.slot, 0);
        assert_eq!(metadata.event_type, EventType::Unknown);
        assert_eq!(metadata.index, "");
        assert_eq!(metadata.program_received_time_ms, 0);
    }

    #[test]
    fn test_new_when_max_values_should_create_instance() {
        // Edge case: maximum values
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "x".repeat(100),
            u64::MAX,
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
            "999".to_string(),
            i64::MAX,
            core,
        );

        assert_eq!(metadata.signature.len(), 100);
        assert_eq!(metadata.slot, u64::MAX);
        assert_eq!(metadata.program_received_time_ms, i64::MAX);
    }

    #[test]
    fn test_new_when_negative_time_should_create_instance() {
        // Edge case: negative time
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Transfer,
            ProtocolType::MarginFi,
            "1".to_string(),
            -1000,
            core,
        );

        assert_eq!(metadata.program_received_time_ms, -1000);
    }

    #[test]
    fn test_set_id_when_valid_id_should_update_core() {
        let core = create_test_core_metadata();
        let mut metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        metadata.set_id("new-id".to_string());
        assert_eq!(metadata.id(), "new-id");
    }

    #[test]
    fn test_set_id_when_empty_id_should_update_core() {
        let core = create_test_core_metadata();
        let mut metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        metadata.set_id("".to_string());
        assert_eq!(metadata.id(), "");
    }

    #[test]
    fn test_id_when_set_should_return_correct_value() {
        let mut core = create_test_core_metadata();
        core.id = "custom-id".to_string();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(metadata.id(), "custom-id");
    }

    #[test]
    fn test_kind_when_accessed_should_return_core_kind() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(metadata.kind(), &EventKind::Transaction);
    }

    #[test]
    fn test_event_kind_when_swap_should_return_swap() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(metadata.event_kind(), EventKind::Swap);
    }

    #[test]
    fn test_event_kind_when_transfer_should_return_transfer() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Transfer,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(metadata.event_kind(), EventKind::Transfer);
    }

    #[test]
    fn test_event_kind_when_liquidate_should_return_custom_liquidation() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Liquidate,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Custom("Liquidate".to_string())
        );
    }

    #[test]
    fn test_event_kind_when_deposit_should_return_transfer() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Deposit,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Transfer
        );
    }

    #[test]
    fn test_event_kind_when_withdraw_should_return_transfer() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Withdraw,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Transfer
        );
    }

    #[test]
    fn test_event_kind_when_borrow_should_return_custom_borrow() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Borrow,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Custom("Borrow".to_string())
        );
    }

    #[test]
    fn test_event_kind_when_repay_should_return_custom_repay() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Repay,
            ProtocolType::MarginFi,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Custom("Repay".to_string())
        );
    }

    #[test]
    fn test_event_kind_when_create_pool_should_return_custom_create_pool() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::CreatePool,
            ProtocolType::RaydiumAmm,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Custom("CreatePool".to_string())
        );
    }

    #[test]
    fn test_event_kind_when_add_liquidity_should_return_liquidity() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::AddLiquidity,
            ProtocolType::OrcaWhirlpool,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Liquidity
        );
    }

    #[test]
    fn test_event_kind_when_remove_liquidity_should_return_liquidity() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::RemoveLiquidity,
            ProtocolType::OrcaWhirlpool,
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Liquidity
        );
    }

    #[test]
    fn test_event_kind_when_unknown_should_return_custom_unknown() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Unknown,
            ProtocolType::default(),
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Custom("Unknown".to_string())
        );
    }

    #[test]
    fn test_event_kind_when_other_variant_should_return_block() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Block, // Explicitly handled variant
            ProtocolType::default(),
            "0".to_string(),
            1000,
            core,
        );

        assert_eq!(
            metadata.event_kind(),
            EventKind::Block
        );
    }

    #[test]
    fn test_core_when_accessed_should_return_reference() {
        let core = create_test_core_metadata();
        let expected_id = core.id.clone();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let core_ref = metadata.core();
        assert_eq!(core_ref.id, expected_id);
    }

    #[test]
    fn test_core_mut_when_accessed_should_return_mutable_reference() {
        let core = create_test_core_metadata();
        let mut metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let core_mut = metadata.core_mut();
        core_mut.id = "modified-id".to_string();

        assert_eq!(metadata.core.id, "modified-id");
    }

    #[test]
    fn test_default_when_created_should_have_empty_values() {
        let metadata = SolanaEventMetadata::default();

        assert_eq!(metadata.signature, "");
        assert_eq!(metadata.slot, 0);
        assert_eq!(metadata.event_type, EventType::default());
        assert_eq!(metadata.protocol_type, ProtocolType::default());
        assert_eq!(metadata.index, "");
        assert_eq!(metadata.program_received_time_ms, 0);
    }

    #[test]
    fn test_default_when_created_should_have_valid_core() {
        let metadata = SolanaEventMetadata::default();

        // Core should be properly initialized
        assert_eq!(metadata.core.source, "solana");
        assert_eq!(metadata.core.kind, EventKind::Transaction);
    }

    #[test]
    fn test_create_metadata_when_valid_inputs_should_create_instance() {
        // Happy path for create_metadata function
        let metadata = create_metadata(
            "event-123".to_string(),
            "signature123".to_string(),
            12345,
            Some(1234567890),
            9876543210,
            "1".to_string(),
            EventType::Swap,
            ProtocolType::Jupiter,
        );

        assert_eq!(metadata.signature, "signature123");
        assert_eq!(metadata.slot, 12345);
        assert_eq!(metadata.event_type, EventType::Swap);
        assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
        assert_eq!(metadata.index, "1");
        assert_eq!(metadata.program_received_time_ms, 9876543210);
        assert_eq!(metadata.core.id, "event-123");
    }

    #[test]
    fn test_create_metadata_when_none_block_time_should_create_instance() {
        // Edge case: None block_time
        let metadata = create_metadata(
            "event-456".to_string(),
            "sig456".to_string(),
            67890,
            None,
            1111111111,
            "2".to_string(),
            EventType::Transfer,
            ProtocolType::MarginFi,
        );

        assert_eq!(metadata.signature, "sig456");
        assert_eq!(metadata.slot, 67890);
        assert_eq!(metadata.event_type, EventType::Transfer);
        assert_eq!(metadata.protocol_type, ProtocolType::MarginFi);
        assert_eq!(metadata.index, "2");
        assert_eq!(metadata.program_received_time_ms, 1111111111);
    }

    #[test]
    fn test_create_metadata_when_empty_strings_should_create_instance() {
        // Edge case: empty strings
        let metadata = create_metadata(
            "".to_string(),
            "".to_string(),
            0,
            Some(0),
            0,
            "".to_string(),
            EventType::Unknown,
            ProtocolType::default(),
        );

        assert_eq!(metadata.signature, "");
        assert_eq!(metadata.slot, 0);
        assert_eq!(metadata.index, "");
        assert_eq!(metadata.core.id, "");
    }

    #[test]
    fn test_create_metadata_when_negative_received_time_should_create_instance() {
        // Edge case: negative received time
        let metadata = create_metadata(
            "event-negative".to_string(),
            "sig-negative".to_string(),
            100,
            Some(1000),
            -5000,
            "3".to_string(),
            EventType::Liquidate,
            ProtocolType::MarginFi,
        );

        assert_eq!(metadata.program_received_time_ms, -5000);
    }

    #[test]
    fn test_clone_when_cloned_should_be_equal() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let cloned = metadata.clone();
        assert_eq!(metadata, cloned);
    }

    #[test]
    fn test_partial_eq_when_same_values_should_be_equal() {
        let core1 = create_test_core_metadata();
        let core2 = create_test_core_metadata();

        let metadata1 = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core1,
        );

        let metadata2 = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core2,
        );

        assert_eq!(metadata1, metadata2);
    }

    #[test]
    fn test_partial_eq_when_different_signatures_should_not_be_equal() {
        let core1 = create_test_core_metadata();
        let core2 = create_test_core_metadata();

        let metadata1 = SolanaEventMetadata::new(
            "sig1".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core1,
        );

        let metadata2 = SolanaEventMetadata::new(
            "sig2".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core2,
        );

        assert_ne!(metadata1, metadata2);
    }

    #[test]
    fn test_debug_when_formatted_should_contain_fields() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let debug_string = format!("{:?}", metadata);
        assert!(debug_string.contains("SolanaEventMetadata"));
        assert!(debug_string.contains("signature"));
        assert!(debug_string.contains("slot"));
        assert!(debug_string.contains("event_type"));
    }

    #[test]
    fn test_serde_serialization_when_serialized_should_succeed() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let serialized = serde_json::to_string(&metadata);
        assert!(serialized.is_ok());

        let json = serialized.unwrap();
        assert!(json.contains("signature"));
        assert!(json.contains("slot"));
        assert!(json.contains("event_type"));
    }

    #[test]
    fn test_serde_deserialization_when_valid_json_should_succeed() {
        let core = create_test_core_metadata();
        let original = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Result<SolanaEventMetadata, _> = serde_json::from_str(&json);

        assert!(deserialized.is_ok());
        let metadata = deserialized.unwrap();
        assert_eq!(metadata.signature, "sig");
        assert_eq!(metadata.slot, 100);
        assert_eq!(metadata.event_type, EventType::Swap);
        assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
    }

    #[test]
    fn test_borsh_serialization_when_serialized_should_succeed() {
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let serialized = borsh::to_vec(&metadata);
        assert!(serialized.is_ok());
        assert!(!serialized.unwrap().is_empty());
    }

    #[test]
    fn test_borsh_deserialization_when_valid_data_should_succeed() {
        let core = create_test_core_metadata();
        let original = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let serialized = borsh::to_vec(&original).unwrap();
        let deserialized: Result<SolanaEventMetadata, _> = borsh::from_slice(&serialized);

        assert!(deserialized.is_ok());
        let metadata = deserialized.unwrap();
        assert_eq!(metadata.signature, "sig");
        assert_eq!(metadata.slot, 100);
        assert_eq!(metadata.event_type, EventType::Swap);
        assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
    }

    #[test]
    fn test_borsh_skip_core_when_serialized_should_exclude_core() {
        // Test that core field is skipped in borsh serialization
        let core = create_test_core_metadata();
        let metadata = SolanaEventMetadata::new(
            "sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            1000,
            core,
        );

        let serialized = borsh::to_vec(&metadata).unwrap();
        let deserialized: SolanaEventMetadata = borsh::from_slice(&serialized).unwrap();

        // Core should be default since it was skipped
        assert_eq!(deserialized.core.id, "");
        assert_eq!(deserialized.core.source, "solana");
    }
}
