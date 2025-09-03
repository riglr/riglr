//! Helper functions for working with Solana-specific EventMetadata
//!
//! This module provides utilities to properly create and access Solana event metadata
//! through the ChainData abstraction from riglr-events-core.

use crate::types::{EventType, ProtocolType};
use chrono::{DateTime, Utc};
use riglr_events_core::prelude::ChainData;
use riglr_events_core::{EventKind, EventMetadata};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;

/// Create core EventMetadata without Solana-specific chain data
pub fn create_core_metadata(
    id: String,
    kind: EventKind,
    source: String,
    block_time: Option<i64>,
) -> EventMetadata {
    let timestamp = block_time
        .and_then(|t| DateTime::from_timestamp(t, 0))
        .unwrap_or_else(Utc::now);
    EventMetadata::with_timestamp(id, kind, source, timestamp)
}

/// Create a new EventMetadata for Solana events with all required fields
/// DEPRECATED: Use create_core_metadata + SolanaEventMetadata::new instead
#[allow(clippy::too_many_arguments)]
#[deprecated(note = "Use create_core_metadata + SolanaEventMetadata::new instead")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::ChainData;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    fn create_test_pubkey() -> Pubkey {
        Pubkey::from_str("11111111111111111111111111111112").unwrap()
    }

    #[test]
    fn test_create_solana_metadata_when_valid_params_should_create_metadata() {
        let metadata = create_solana_metadata(
            "test_id".to_string(),
            EventKind::Transaction,
            "test_source".to_string(),
            123,
            Some("test_signature".to_string()),
            Some(create_test_pubkey()),
            Some(5),
            Some(1640995200),
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        assert_eq!(metadata.id, "test_id");
        assert_eq!(metadata.kind, EventKind::Transaction);
        assert_eq!(metadata.source, "test_source");

        match &metadata.chain_data {
            Some(ChainData::Solana {
                slot,
                signature,
                program_id,
                instruction_index,
                block_time,
                protocol_data,
            }) => {
                assert_eq!(*slot, 123);
                assert_eq!(signature.as_ref().unwrap(), "test_signature");
                assert_eq!(program_id.as_ref().unwrap(), &create_test_pubkey());
                assert_eq!(*instruction_index, Some(5));
                assert_eq!(*block_time, Some(1640995200));
                assert!(protocol_data.is_some());
            }
            _ => panic!("Expected Solana chain data"),
        }
    }

    #[test]
    fn test_create_solana_metadata_when_none_optionals_should_create_metadata() {
        let metadata = create_solana_metadata(
            "test_id".to_string(),
            EventKind::Block,
            "test_source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        match &metadata.chain_data {
            Some(ChainData::Solana {
                signature,
                program_id,
                instruction_index,
                block_time,
                ..
            }) => {
                assert!(signature.is_none());
                assert!(program_id.is_none());
                assert!(instruction_index.is_none());
                assert!(block_time.is_none());
            }
            _ => panic!("Expected Solana chain data"),
        }
    }

    #[test]
    fn test_get_slot_when_solana_metadata_should_return_slot() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            42,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let slot = get_slot(&metadata);
        assert_eq!(slot, Some(42));
    }

    #[test]
    fn test_get_slot_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let slot = get_slot(&metadata);
        assert_eq!(slot, None);
    }

    #[test]
    fn test_get_signature_when_solana_metadata_with_signature_should_return_signature() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            Some("test_signature".to_string()),
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let signature = get_signature(&metadata);
        assert_eq!(signature, Some("test_signature"));
    }

    #[test]
    fn test_get_signature_when_solana_metadata_without_signature_should_return_none() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let signature = get_signature(&metadata);
        assert_eq!(signature, None);
    }

    #[test]
    fn test_get_signature_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let signature = get_signature(&metadata);
        assert_eq!(signature, None);
    }

    #[test]
    fn test_get_program_id_when_solana_metadata_with_program_id_should_return_program_id() {
        let test_pubkey = create_test_pubkey();
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            Some(test_pubkey),
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let program_id = get_program_id(&metadata);
        assert_eq!(program_id, Some(&test_pubkey));
    }

    #[test]
    fn test_get_program_id_when_solana_metadata_without_program_id_should_return_none() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let program_id = get_program_id(&metadata);
        assert_eq!(program_id, None);
    }

    #[test]
    fn test_get_program_id_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let program_id = get_program_id(&metadata);
        assert_eq!(program_id, None);
    }

    #[test]
    fn test_get_instruction_index_when_solana_metadata_with_index_should_return_index() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            Some(7),
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let index = get_instruction_index(&metadata);
        assert_eq!(index, Some(7));
    }

    #[test]
    fn test_get_instruction_index_when_solana_metadata_without_index_should_return_none() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let index = get_instruction_index(&metadata);
        assert_eq!(index, None);
    }

    #[test]
    fn test_get_instruction_index_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let index = get_instruction_index(&metadata);
        assert_eq!(index, None);
    }

    #[test]
    fn test_get_block_time_when_solana_metadata_with_block_time_should_return_block_time() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            Some(1640995200),
            ProtocolType::default(),
            EventType::default(),
        );

        let block_time = get_block_time(&metadata);
        assert_eq!(block_time, Some(1640995200));
    }

    #[test]
    fn test_get_block_time_when_solana_metadata_without_block_time_should_return_none() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::default(),
            EventType::default(),
        );

        let block_time = get_block_time(&metadata);
        assert_eq!(block_time, None);
    }

    #[test]
    fn test_get_block_time_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let block_time = get_block_time(&metadata);
        assert_eq!(block_time, None);
    }

    #[test]
    fn test_get_protocol_type_when_solana_metadata_should_return_protocol_type() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, Some(ProtocolType::Jupiter));
    }

    #[test]
    fn test_get_protocol_type_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, None);
    }

    #[test]
    fn test_get_protocol_type_when_invalid_protocol_data_should_return_none() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        metadata = metadata.with_chain_data(ChainData::Solana {
            slot: 0,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: Some(json!({"invalid": "data"})),
        });

        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, None);
    }

    #[test]
    fn test_get_event_type_when_solana_metadata_should_return_event_type() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, Some(EventType::Swap));
    }

    #[test]
    fn test_get_event_type_when_no_chain_data_should_return_none() {
        let metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, None);
    }

    #[test]
    fn test_get_event_type_when_invalid_protocol_data_should_return_none() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        metadata = metadata.with_chain_data(ChainData::Solana {
            slot: 0,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: Some(json!({"invalid": "data"})),
        });

        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, None);
    }

    #[test]
    fn test_set_protocol_type_when_solana_metadata_should_update_protocol_type() {
        let mut metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        set_protocol_type(&mut metadata, ProtocolType::RaydiumAmm);
        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, Some(ProtocolType::RaydiumAmm));
    }

    #[test]
    fn test_set_protocol_type_when_no_chain_data_should_not_panic() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        set_protocol_type(&mut metadata, ProtocolType::Jupiter);
        // Should not panic, but also won't set anything
        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, None);
    }

    #[test]
    fn test_set_protocol_type_when_empty_protocol_data_should_create_object() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        metadata = metadata.with_chain_data(ChainData::Solana {
            slot: 0,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: None,
        });

        set_protocol_type(&mut metadata, ProtocolType::Jupiter);
        let protocol_type = get_protocol_type(&metadata);
        assert_eq!(protocol_type, Some(ProtocolType::Jupiter));
    }

    #[test]
    fn test_set_event_type_when_solana_metadata_should_update_event_type() {
        let mut metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            0,
            None,
            None,
            None,
            None,
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        set_event_type(&mut metadata, EventType::Transfer);
        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, Some(EventType::Transfer));
    }

    #[test]
    fn test_set_event_type_when_no_chain_data_should_not_panic() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        set_event_type(&mut metadata, EventType::Swap);
        // Should not panic, but also won't set anything
        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, None);
    }

    #[test]
    fn test_set_event_type_when_empty_protocol_data_should_create_object() {
        let mut metadata = EventMetadata::new(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
        );
        metadata = metadata.with_chain_data(ChainData::Solana {
            slot: 0,
            signature: None,
            program_id: None,
            instruction_index: None,
            block_time: None,
            protocol_data: None,
        });

        set_event_type(&mut metadata, EventType::Swap);
        let event_type = get_event_type(&metadata);
        assert_eq!(event_type, Some(EventType::Swap));
    }

    #[test]
    fn test_edge_cases_with_all_protocol_types() {
        let protocol_types = vec![
            ProtocolType::OrcaWhirlpool,
            ProtocolType::MeteoraDlmm,
            ProtocolType::MarginFi,
            ProtocolType::Bonk,
            ProtocolType::PumpSwap,
            ProtocolType::RaydiumAmm,
            ProtocolType::RaydiumAmmV4,
            ProtocolType::RaydiumClmm,
            ProtocolType::RaydiumCpmm,
            ProtocolType::Jupiter,
            ProtocolType::Other("Custom".to_string()),
        ];

        for protocol_type in protocol_types {
            let metadata = create_solana_metadata(
                "test".to_string(),
                EventKind::Transaction,
                "source".to_string(),
                0,
                None,
                None,
                None,
                None,
                protocol_type.clone(),
                EventType::Swap,
            );

            let retrieved_protocol_type = get_protocol_type(&metadata);
            assert_eq!(retrieved_protocol_type, Some(protocol_type));
        }
    }

    #[test]
    fn test_edge_cases_with_various_event_types() {
        let event_types = vec![
            EventType::Swap,
            EventType::AddLiquidity,
            EventType::RemoveLiquidity,
            EventType::Transfer,
            EventType::Unknown,
            EventType::RaydiumSwap,
            EventType::PumpSwapBuy,
            EventType::BonkBuyExactIn,
        ];

        for event_type in event_types {
            let metadata = create_solana_metadata(
                "test".to_string(),
                EventKind::Transaction,
                "source".to_string(),
                0,
                None,
                None,
                None,
                None,
                ProtocolType::Jupiter,
                event_type.clone(),
            );

            let retrieved_event_type = get_event_type(&metadata);
            assert_eq!(retrieved_event_type, Some(event_type));
        }
    }

    #[test]
    fn test_edge_cases_with_max_values() {
        let metadata = create_solana_metadata(
            "test".to_string(),
            EventKind::Transaction,
            "source".to_string(),
            u64::MAX,
            Some("very_long_signature_string_that_might_be_used_in_production".to_string()),
            Some(create_test_pubkey()),
            Some(usize::MAX),
            Some(i64::MAX),
            ProtocolType::Jupiter,
            EventType::Swap,
        );

        assert_eq!(get_slot(&metadata), Some(u64::MAX));
        assert_eq!(get_instruction_index(&metadata), Some(usize::MAX));
        assert_eq!(get_block_time(&metadata), Some(i64::MAX));
    }

    #[test]
    fn test_edge_cases_with_empty_strings() {
        let metadata = create_solana_metadata(
            "".to_string(),
            EventKind::Transaction,
            "".to_string(),
            0,
            Some("".to_string()),
            None,
            None,
            None,
            ProtocolType::Other("".to_string()),
            EventType::Unknown,
        );

        assert_eq!(metadata.id, "");
        assert_eq!(metadata.source, "");
        assert_eq!(get_signature(&metadata), Some(""));
    }
}
