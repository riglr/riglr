//! Integration tests for riglr-solana-events crate
//!
//! These tests verify that the E0107 errors have been fixed and that
//! the EventMetadata type system is working correctly.

use riglr_events_core::EventKind;
use riglr_solana_events::solana_metadata::SolanaEventMetadata;
use riglr_solana_events::types::{EventType, ProtocolType};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[test]
fn test_event_metadata_creation() {
    // Test creating EventMetadata (which is now SolanaEventMetadata) using the helper
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        12345,
        1640995200,
        ProtocolType::Jupiter,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(metadata.signature, "test-signature");
    assert_eq!(metadata.slot, 12345);
    assert_eq!(metadata.event_type, EventType::Swap);
    assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
    assert_eq!(metadata.index, "0");
    assert_eq!(metadata.program_received_time_ms, 1640995200000);
}

#[test]
fn test_metadata_helpers_create_core_metadata() {
    let core_metadata = riglr_solana_events::metadata_helpers::create_core_metadata(
        "test-id".to_string(),
        EventKind::Swap,
        "solana".to_string(),
        Some(1640995200),
    );

    // Test that the core metadata was created successfully without duplication
    assert_eq!(core_metadata.id, "test-id");
    assert_eq!(core_metadata.kind, EventKind::Swap);
    assert_eq!(core_metadata.source, "solana");

    // Verify that chain_data is not populated (eliminating duplication)
    assert!(core_metadata.chain_data.is_none());
}

#[test]
fn test_solana_event_metadata_wrapper() {
    // Test the new pattern: create core metadata, then wrap in SolanaEventMetadata
    use riglr_solana_events::solana_metadata::SolanaEventMetadata;

    let core_metadata = riglr_solana_events::metadata_helpers::create_core_metadata(
        "test-id".to_string(),
        EventKind::Swap,
        "solana".to_string(),
        Some(1640995200),
    );

    let solana_metadata = SolanaEventMetadata::new(
        "test-signature".to_string(),
        12345,
        EventType::Swap,
        ProtocolType::Jupiter,
        "0".to_string(),
        1640995200000,
        core_metadata,
    );

    // Test that both core and Solana-specific fields are accessible
    assert_eq!(solana_metadata.id(), "test-id");
    assert_eq!(solana_metadata.signature, "test-signature");
    assert_eq!(solana_metadata.slot, 12345);
    assert_eq!(solana_metadata.event_type, EventType::Swap);
    assert_eq!(solana_metadata.protocol_type, ProtocolType::Jupiter);

    // Verify no duplication: chain_data should be empty
    assert!(solana_metadata.core().chain_data.is_none());
}

#[test]
fn test_bonk_parser_metadata_creation() {
    // Test that the previously problematic bonk parser code now works
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        12345,
        1640995200,
        ProtocolType::Bonk,
        EventType::BonkBuyExactIn,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(metadata.event_type, EventType::BonkBuyExactIn);
    assert_eq!(metadata.protocol_type, ProtocolType::Bonk);
}

#[test]
fn test_marginfi_parser_metadata_creation() {
    // Test that the previously problematic marginfi parser code now works
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        12345,
        1640995200,
        ProtocolType::MarginFi,
        EventType::AddLiquidity,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(metadata.event_type, EventType::AddLiquidity);
    assert_eq!(metadata.protocol_type, ProtocolType::MarginFi);
}

#[test]
fn test_pumpswap_parser_metadata_creation() {
    // Test that the previously problematic pumpswap parser code now works
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        100,
        1640995200,
        ProtocolType::PumpSwap,
        EventType::PumpSwapBuy,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(metadata.event_type, EventType::PumpSwapBuy);
    assert_eq!(metadata.protocol_type, ProtocolType::PumpSwap);
}

#[test]
fn test_enrichment_pipeline_metadata_creation() {
    // Test that the previously problematic enrichment pipeline code now works
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    // Jupiter test
    let jupiter_metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test_sig".to_string(),
        100,
        1640995200,
        ProtocolType::Jupiter,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(jupiter_metadata.protocol_type, ProtocolType::Jupiter);

    // Raydium AMM V4 test
    let raydium_metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test_sig".to_string(),
        100,
        1640995200,
        ProtocolType::RaydiumAmmV4,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(raydium_metadata.protocol_type, ProtocolType::RaydiumAmmV4);

    // Orca Whirlpool test
    let orca_metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test_sig".to_string(),
        100,
        1640995200,
        ProtocolType::OrcaWhirlpool,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    assert_eq!(orca_metadata.protocol_type, ProtocolType::OrcaWhirlpool);
}

#[test]
fn test_all_protocol_types() {
    // Test that all protocol types can be used
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
        ProtocolType::Raydium,
        ProtocolType::Jupiter,
        ProtocolType::Serum,
        ProtocolType::Other("Custom".to_string()),
    ];

    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    for protocol_type in protocol_types {
        let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
            "test-id".to_string(),
            "test-signature".to_string(),
            12345,
            1640995200,
            protocol_type.clone(),
            EventType::Swap,
            program_id,
            "0".to_string(),
            1640995200000,
        );

        assert_eq!(metadata.protocol_type, protocol_type);
    }
}

#[test]
fn test_all_event_types() {
    // Test that all event types can be used
    let event_types = vec![
        EventType::Swap,
        EventType::AddLiquidity,
        EventType::RemoveLiquidity,
        EventType::Borrow,
        EventType::Repay,
        EventType::Liquidate,
        EventType::Transfer,
        EventType::Mint,
        EventType::Burn,
        EventType::CreatePool,
        EventType::UpdatePool,
        EventType::Transaction,
        EventType::Block,
        EventType::ContractEvent,
        EventType::PriceUpdate,
        EventType::OrderBook,
        EventType::Trade,
        EventType::FeeUpdate,
        EventType::BonkBuyExactIn,
        EventType::PumpSwapBuy,
        EventType::RaydiumSwap,
        EventType::Unknown,
    ];

    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    for event_type in event_types {
        let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
            "test-id".to_string(),
            "test-signature".to_string(),
            12345,
            1640995200,
            ProtocolType::Jupiter,
            event_type.clone(),
            program_id,
            "0".to_string(),
            1640995200000,
        );

        assert_eq!(metadata.event_type, event_type);
    }
}

#[test]
fn test_solana_event_metadata_borsh_serialization() {
    // Test that SolanaEventMetadata can be serialized/deserialized with Borsh
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        12345,
        1640995200,
        ProtocolType::Jupiter,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    // Test Borsh serialization
    let serialized = borsh::to_vec(&metadata).expect("Should serialize");
    let deserialized: SolanaEventMetadata =
        borsh::from_slice(&serialized).expect("Should deserialize");

    assert_eq!(metadata.signature, deserialized.signature);
    assert_eq!(metadata.slot, deserialized.slot);
    assert_eq!(metadata.event_type, deserialized.event_type);
    assert_eq!(metadata.protocol_type, deserialized.protocol_type);
}

#[test]
fn test_solana_event_metadata_json_serialization() {
    // Test that SolanaEventMetadata can be serialized/deserialized with JSON
    let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

    let metadata = riglr_solana_events::types::metadata_helpers::create_solana_metadata(
        "test-id".to_string(),
        "test-signature".to_string(),
        12345,
        1640995200,
        ProtocolType::Jupiter,
        EventType::Swap,
        program_id,
        "0".to_string(),
        1640995200000,
    );

    // Test JSON serialization
    let json = serde_json::to_string(&metadata).expect("Should serialize to JSON");
    let deserialized: SolanaEventMetadata =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(metadata.signature, deserialized.signature);
    assert_eq!(metadata.slot, deserialized.slot);
    assert_eq!(metadata.event_type, deserialized.event_type);
    assert_eq!(metadata.protocol_type, deserialized.protocol_type);
}

#[test]
fn test_direct_solana_event_metadata_construction() {
    // Test direct construction using SolanaEventMetadata::new
    let core = riglr_events_core::EventMetadata::new(
        "test-id".to_string(),
        EventKind::Swap,
        "solana".to_string(),
    );

    let metadata = riglr_solana_events::solana_metadata::SolanaEventMetadata::new(
        "test-signature".to_string(),
        12345,
        EventType::Swap,
        ProtocolType::Jupiter,
        "0".to_string(),
        1640995200000,
        core,
    );

    assert_eq!(metadata.signature, "test-signature");
    assert_eq!(metadata.slot, 12345);
    assert_eq!(metadata.event_type, EventType::Swap);
    assert_eq!(metadata.protocol_type, ProtocolType::Jupiter);
    assert_eq!(metadata.index, "0");
    assert_eq!(metadata.program_received_time_ms, 1640995200000);
}

#[test]
fn test_event_metadata_id_accessor() {
    // Test that we can access the ID properly
    let core = riglr_events_core::EventMetadata::new(
        "test-id".to_string(),
        EventKind::Swap,
        "solana".to_string(),
    );

    let metadata = riglr_solana_events::solana_metadata::SolanaEventMetadata::new(
        "test-signature".to_string(),
        12345,
        EventType::Swap,
        ProtocolType::Jupiter,
        "0".to_string(),
        1640995200000,
        core,
    );

    // Access ID using the method
    assert_eq!(metadata.id(), "test-id");

    // Test that we can access the kind
    assert_eq!(metadata.kind(), &EventKind::Swap);
}
