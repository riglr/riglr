//! Tests to verify metadata is not duplicated

#![allow(clippy::expect_used)]

use riglr_events_core::EventKind;
use riglr_solana_events::{
    metadata_helpers::create_core_metadata,
    solana_metadata::SolanaEventMetadata,
    types::{EventType, ProtocolType},
};

#[test]
fn test_metadata_creation_no_duplication() {
    // Create metadata using the recommended pattern
    let core = create_core_metadata(
        "test_id".to_string(),
        EventKind::Transaction,
        "solana".to_string(),
        Some(1_234_567_890),
    );

    // Verify core has no chain_data (avoiding duplication)
    assert!(
        core.chain_data.is_none(),
        "Core metadata should not have chain_data to avoid duplication"
    );

    // Create SolanaEventMetadata wrapper
    let solana_metadata = SolanaEventMetadata::new(
        "test_signature".to_string(),
        12345,
        EventType::Swap,
        ProtocolType::Other("Test".to_string()),
        "0".to_string(),
        1_234_567_890_000,
        core,
    );

    // Verify Solana-specific fields are in the wrapper
    assert_eq!(solana_metadata.signature, "test_signature");
    assert_eq!(solana_metadata.slot, 12345);

    // Verify core metadata is accessible and has no chain_data
    assert!(
        solana_metadata.core().chain_data.is_none(),
        "Core metadata within SolanaEventMetadata should not have chain_data to avoid duplication"
    );
}

#[test]
fn test_metadata_helpers_create_solana_metadata_no_duplication() {
    use core::str::FromStr;
    use riglr_solana_events::types::metadata_helpers;
    use solana_sdk::pubkey::Pubkey;

    // Use the helper function that's being used in parsers
    let metadata = metadata_helpers::create_solana_metadata(
        "test_id".to_string(),
        "test_signature".to_string(),
        12345,
        1_234_567_890,
        ProtocolType::Other("Test".to_string()),
        EventType::Swap,
        Pubkey::from_str("11111111111111111111111111111112")
            .expect("Should parse valid test Pubkey"),
        "0".to_string(),
        1_234_567_890_000,
    );

    // Verify the helper creates the correct structure without duplication
    assert_eq!(metadata.signature, "test_signature");
    assert_eq!(metadata.slot, 12345);

    // Most importantly: verify core has no chain_data (no duplication)
    assert!(
        metadata.core().chain_data.is_none(),
        "Helper function should create metadata without chain_data duplication"
    );
}
