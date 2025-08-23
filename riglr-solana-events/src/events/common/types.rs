// Re-export types from the main types module
pub use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexport_protocol_type_works() {
        // Test that ProtocolType is accessible through re-export
        let protocol = ProtocolType::OrcaWhirlpool;
        assert_eq!(protocol, ProtocolType::OrcaWhirlpool);
    }

    #[test]
    fn test_reexport_event_type_works() {
        // Test that EventType is accessible through re-export
        let event = EventType::Swap;
        assert_eq!(event, EventType::Swap);
    }

    #[test]
    fn test_reexport_transfer_data_works() {
        // Test that TransferData is accessible through re-export
        use solana_sdk::pubkey::Pubkey;
        use std::str::FromStr;

        let source = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let destination = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let transfer_data = TransferData {
            source,
            destination,
            mint: None,
            amount: 100,
        };

        assert_eq!(transfer_data.amount, 100);
    }

    #[test]
    fn test_reexport_swap_data_works() {
        // Test that SwapData is accessible through re-export
        use solana_sdk::pubkey::Pubkey;
        use std::str::FromStr;

        let input_mint = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let output_mint = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let swap_data = SwapData {
            input_mint,
            output_mint,
            amount_in: 1000,
            amount_out: 950,
        };

        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.amount_out, 950);
    }

    #[test]
    fn test_reexport_stream_metadata_works() {
        // Test that StreamMetadata is accessible through re-export
        let stream_metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: std::time::SystemTime::UNIX_EPOCH,
            sequence_number: Some(1),
            custom_data: None,
        };

        assert_eq!(stream_metadata.stream_source, "test");
        assert_eq!(stream_metadata.sequence_number, Some(1));
    }

    #[test]
    fn test_reexport_metadata_helpers_works() {
        // Test that metadata_helpers module is accessible through re-export
        use solana_sdk::pubkey::Pubkey;
        use std::str::FromStr;

        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();

        let metadata = metadata_helpers::create_solana_metadata(
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
    }
}
