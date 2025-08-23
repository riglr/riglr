//! Common utilities and types shared across protocol parsers.
//!
//! This module provides core functionality used by all Solana event parsers including:
//! - Utility functions for parsing various data types from byte arrays
//! - Common type definitions and re-exports
//! - Shared helper functions for instruction decoding and data extraction

/// Re-exported types from the main crate
pub mod types;

/// Parsing utility functions for bytes, integers, pubkeys, and other common operations
pub mod utils;

pub use types::*;
pub use utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_module_types_accessible() {
        // Test that types module is accessible
        let protocol = ProtocolType::OrcaWhirlpool;
        assert_eq!(protocol, ProtocolType::OrcaWhirlpool);

        let event = EventType::Swap;
        assert_eq!(event, EventType::Swap);
    }

    #[test]
    fn test_module_utils_accessible() {
        // Test that utils functions are accessible through re-export
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = read_u64_le(&data, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x0807060504030201);
    }

    #[test]
    fn test_types_reexport_transfer_data() {
        // Test TransferData is accessible through re-export
        let source = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let destination = Pubkey::from_str("11111111111111111111111111111113").unwrap();

        let transfer_data = TransferData {
            source,
            destination,
            mint: None,
            amount: 100,
        };

        assert_eq!(transfer_data.amount, 100);
        assert_eq!(transfer_data.source, source);
        assert_eq!(transfer_data.destination, destination);
        assert_eq!(transfer_data.mint, None);
    }

    #[test]
    fn test_types_reexport_swap_data() {
        // Test SwapData is accessible through re-export
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
        assert_eq!(swap_data.input_mint, input_mint);
        assert_eq!(swap_data.output_mint, output_mint);
    }

    #[test]
    fn test_utils_reexport_read_functions() {
        // Test that all read functions are accessible through re-export
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        // Test read_u64_le
        let u64_result = read_u64_le(&data, 0);
        assert!(u64_result.is_ok());
        assert_eq!(u64_result.unwrap(), u64::MAX);

        // Test read_u128_le
        let u128_result = read_u128_le(&data, 0);
        assert!(u128_result.is_ok());
        assert_eq!(u128_result.unwrap(), u128::MAX);

        // Test read_u32_le
        let u32_result = read_u32_le(&data, 0);
        assert!(u32_result.is_ok());
        assert_eq!(u32_result.unwrap(), u32::MAX);

        // Test read_i32_le (negative value)
        let i32_result = read_i32_le(&data, 0);
        assert!(i32_result.is_ok());
        assert_eq!(i32_result.unwrap(), -1);

        // Test read_u8_le
        let u8_result = read_u8_le(&data, 0);
        assert!(u8_result.is_ok());
        assert_eq!(u8_result.unwrap(), 255);
    }

    #[test]
    fn test_utils_reexport_parse_functions() {
        // Test that all parse functions are accessible through re-export
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];

        // Test parse_u64_le
        let u64_result = parse_u64_le(&data);
        assert!(u64_result.is_ok());

        // Test parse_u128_le
        let u128_result = parse_u128_le(&data);
        assert!(u128_result.is_ok());

        // Test parse_u32_le
        let u32_result = parse_u32_le(&data);
        assert!(u32_result.is_ok());

        // Test parse_u16_le
        let u16_result = parse_u16_le(&data);
        assert!(u16_result.is_ok());
    }

    #[test]
    fn test_utils_reexport_pubkey_functions() {
        // Test pubkey parsing functions are accessible through re-export
        let pubkey_bytes = [1u8; 32];
        let result = parse_pubkey_from_bytes(&pubkey_bytes);
        assert!(result.is_ok());

        let pubkey = result.unwrap();
        assert_eq!(pubkey.to_bytes(), pubkey_bytes);
    }

    #[test]
    fn test_utils_reexport_discriminator_functions() {
        // Test discriminator functions are accessible through re-export
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];

        // Test extract_discriminator
        let discriminator = extract_discriminator(&data, 3);
        assert!(discriminator.is_some());
        assert_eq!(discriminator.unwrap(), vec![0x01, 0x02, 0x03]);

        // Test has_discriminator
        let check_discriminator = [0x01, 0x02, 0x03];
        assert!(has_discriminator(&data, &check_discriminator));

        let wrong_discriminator = [0x01, 0x02, 0x04];
        assert!(!has_discriminator(&data, &wrong_discriminator));
    }

    #[test]
    fn test_utils_reexport_token_amount_functions() {
        // Test token amount functions are accessible through re-export
        let amount = 1_000_000_000u64; // 1 token with 9 decimals
        let formatted = format_token_amount(amount, 9);
        assert_eq!(formatted, 1.0);

        let converted_back = to_token_amount(1.0, 9);
        assert_eq!(converted_back, 1_000_000_000u64);
    }

    #[test]
    fn test_utils_reexport_price_impact_function() {
        // Test price impact calculation is accessible through re-export
        let impact = calculate_price_impact(1000, 900, 10000, 10000);
        assert!(impact > 0.0);
        assert!(impact < 100.0);

        // Test edge case with zero reserves
        let zero_impact = calculate_price_impact(1000, 900, 0, 0);
        assert_eq!(zero_impact, 0.0);
    }

    #[test]
    fn test_utils_reexport_account_keys_function() {
        // Test account keys extraction is accessible through re-export
        let accounts = vec![
            Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            Pubkey::from_str("11111111111111111111111111111114").unwrap(),
        ];
        let indices = vec![0, 2];

        let result = extract_account_keys(&accounts, &indices);
        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], accounts[0]);
        assert_eq!(keys[1], accounts[2]);
    }

    #[test]
    fn test_utils_reexport_base58_functions() {
        // Test base58 encoding/decoding functions are accessible through re-export
        let data = b"hello world";
        let encoded = encode_base58(data);
        assert!(!encoded.is_empty());

        let decoded = decode_base58(&encoded);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap(), data.to_vec());
    }

    #[test]
    fn test_utils_reexport_time_function() {
        // Test time conversion function is accessible through re-export
        let time = std::time::SystemTime::UNIX_EPOCH;
        let millis = system_time_to_millis(time);
        assert_eq!(millis, 0);

        let now = std::time::SystemTime::now();
        let now_millis = system_time_to_millis(now);
        assert!(now_millis > 0);
    }

    #[test]
    fn test_utils_reexport_option_bool_function() {
        // Test option bool parsing is accessible through re-export
        let data = [0u8]; // None variant
        let mut offset = 0;
        let result = read_option_bool(&data, &mut offset);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
        assert_eq!(offset, 1);

        let data_some_true = [1u8, 1u8]; // Some(true)
        let mut offset = 0;
        let result = read_option_bool(&data_some_true, &mut offset);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(true));
        assert_eq!(offset, 2);

        let data_some_false = [1u8, 0u8]; // Some(false)
        let mut offset = 0;
        let result = read_option_bool(&data_some_false, &mut offset);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(false));
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_types_reexport_stream_metadata() {
        // Test StreamMetadata is accessible through re-export
        let stream_metadata = StreamMetadata {
            stream_source: "test_source".to_string(),
            received_at: std::time::SystemTime::UNIX_EPOCH,
            sequence_number: Some(42),
            custom_data: Some(serde_json::json!({"key": "value"})),
        };

        assert_eq!(stream_metadata.stream_source, "test_source");
        assert_eq!(
            stream_metadata.received_at,
            std::time::SystemTime::UNIX_EPOCH
        );
        assert_eq!(stream_metadata.sequence_number, Some(42));
        assert!(stream_metadata.custom_data.is_some());
    }

    #[test]
    fn test_types_reexport_protocol_type_variants() {
        // Test all ProtocolType variants are accessible
        let jupiter = ProtocolType::Jupiter;
        let orca = ProtocolType::OrcaWhirlpool;
        let raydium = ProtocolType::Raydium;
        let serum = ProtocolType::Serum;

        assert_eq!(jupiter, ProtocolType::Jupiter);
        assert_eq!(orca, ProtocolType::OrcaWhirlpool);
        assert_eq!(raydium, ProtocolType::Raydium);
        assert_eq!(serum, ProtocolType::Serum);

        // Test they are not equal to each other
        assert_ne!(jupiter, orca);
        assert_ne!(orca, raydium);
        assert_ne!(raydium, serum);
    }

    #[test]
    fn test_types_reexport_event_type_variants() {
        // Test all EventType variants are accessible
        let swap = EventType::Swap;
        let transfer = EventType::Transfer;
        let liquidity_provision = EventType::LiquidityProvision;
        let liquidity_removal = EventType::LiquidityRemoval;

        assert_eq!(swap, EventType::Swap);
        assert_eq!(transfer, EventType::Transfer);
        assert_eq!(liquidity_provision, EventType::LiquidityProvision);
        assert_eq!(liquidity_removal, EventType::LiquidityRemoval);

        // Test they are not equal to each other
        assert_ne!(swap, transfer);
        assert_ne!(transfer, liquidity_provision);
        assert_ne!(liquidity_provision, liquidity_removal);
    }
}
