pub mod events;
pub mod parsers;

pub use events::*;
pub use parsers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::metadata_helpers::create_solana_metadata;
    use crate::types::{EventType, ProtocolType};
    type EventMetadata = SolanaEventMetadata;
    use solana_sdk::pubkey::Pubkey;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::tempdir;

    // Mock implementation of ByteSliceEventParser for testing
    #[derive(Debug)]
    struct MockParser {
        protocol: ProtocolType,
        can_parse_result: bool,
    }

    impl ByteSliceEventParser for MockParser {
        fn parse_from_slice<'a>(
            &self,
            data: &'a [u8],
            metadata: EventMetadata,
        ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
            if data.is_empty() {
                return Err(ParseError::InvalidInstructionData("Empty data".to_string()));
            }

            let event = ZeroCopyEvent::new_borrowed(metadata, data);
            Ok(vec![event])
        }

        fn can_parse(&self, _data: &[u8]) -> bool {
            self.can_parse_result
        }

        fn protocol_type(&self) -> ProtocolType {
            self.protocol.clone()
        }
    }

    fn create_test_metadata() -> EventMetadata {
        create_solana_metadata(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            1234567890,
            ProtocolType::Jupiter,
            EventType::Swap,
            Pubkey::default(),
            "0".to_string(),
            0,
        )
    }

    // Tests for ZeroCopyEvent
    #[test]
    fn test_zero_copy_event_new_borrowed() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata.clone(), data);

        assert_eq!(event.raw_data(), data);
        assert!(event.parsed_data.is_none());
        assert!(event.json_data.is_none());
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_zero_copy_event_new_owned() {
        let data = b"test data".to_vec();
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_owned(metadata.clone(), data.clone());

        assert_eq!(event.raw_data(), data.as_slice());
        assert!(event.parsed_data.is_none());
        assert!(event.json_data.is_none());
    }

    #[test]
    fn test_zero_copy_event_raw_data() {
        let data = b"test raw data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata, data);

        assert_eq!(event.raw_data(), data);
    }

    #[test]
    fn test_zero_copy_event_set_and_get_parsed_data() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let mut event = ZeroCopyEvent::new_borrowed(metadata, data);

        // Test setting and getting parsed data
        let test_string = "parsed data".to_string();
        event.set_parsed_data(test_string.clone());

        let retrieved = event.get_parsed_data::<String>().unwrap();
        assert_eq!(retrieved, &test_string);

        // Test getting wrong type returns None
        assert!(event.get_parsed_data::<u32>().is_none());
    }

    #[test]
    fn test_zero_copy_event_get_parsed_data_when_none() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata, data);

        assert!(event.get_parsed_data::<String>().is_none());
    }

    #[test]
    fn test_zero_copy_event_set_and_get_json_data() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let mut event = ZeroCopyEvent::new_borrowed(metadata, data);

        let json_value = serde_json::json!({"key": "value"});
        event.set_json_data(json_value.clone());

        assert_eq!(event.get_json_data(), Some(&json_value));
    }

    #[test]
    fn test_zero_copy_event_get_json_data_when_none() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata, data);

        assert!(event.get_json_data().is_none());
    }

    #[test]
    fn test_zero_copy_event_to_owned() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let mut event = ZeroCopyEvent::new_borrowed(metadata.clone(), data);

        // Set some data
        event.set_parsed_data("test".to_string());
        event.set_json_data(serde_json::json!({"test": true}));

        let owned_event = event.to_owned();
        assert_eq!(owned_event.raw_data(), data);
        assert_eq!(owned_event.id(), "test_id");
        assert!(owned_event.get_parsed_data::<String>().is_some());
        assert!(owned_event.get_json_data().is_some());
    }

    #[test]
    fn test_zero_copy_event_helper_methods() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata, data);

        assert_eq!(event.id(), "test_id");
        assert_eq!(event.event_type(), EventType::Swap);
        assert_eq!(event.signature(), "test_signature");
        assert_eq!(event.slot(), 12345);
        assert_eq!(event.protocol_type(), ProtocolType::Jupiter);
        assert_eq!(event.index(), "0");
        assert!(event.block_number().is_some());
        assert_eq!(event.block_number().unwrap(), 12345);
    }

    #[test]
    fn test_zero_copy_event_timestamp() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let event = ZeroCopyEvent::new_borrowed(metadata, data);

        let timestamp = event.timestamp();
        // Just verify it's a valid SystemTime
        assert!(timestamp.duration_since(std::time::UNIX_EPOCH).is_ok());
    }

    // Tests for ZeroCopySwapEvent
    #[test]
    fn test_zero_copy_swap_event_new() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let base = ZeroCopyEvent::new_borrowed(metadata, data);
        let swap_event = ZeroCopySwapEvent::new(base);

        assert_eq!(swap_event.base.raw_data(), data);
        // These fields are private, but we can test the public methods which should return None
        // since the base parse methods are stubs that return None
        let mut swap_event_mut = swap_event.clone();
        assert!(swap_event_mut.input_mint().is_none());
        assert!(swap_event_mut.output_mint().is_none());
        assert!(swap_event_mut.amount_in().is_none());
        assert!(swap_event_mut.amount_out().is_none());
    }

    #[test]
    fn test_zero_copy_swap_event_lazy_parsing() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let base = ZeroCopyEvent::new_borrowed(metadata, data);
        let mut swap_event = ZeroCopySwapEvent::new(base);

        // Test that calling getter methods returns None (since parsing methods return None)
        assert!(swap_event.input_mint().is_none());
        assert!(swap_event.output_mint().is_none());
        assert!(swap_event.amount_in().is_none());
        assert!(swap_event.amount_out().is_none());

        // Test that calling again uses cached values
        assert!(swap_event.input_mint().is_none());
        assert!(swap_event.output_mint().is_none());
        assert!(swap_event.amount_in().is_none());
        assert!(swap_event.amount_out().is_none());
    }

    // Tests for ZeroCopyLiquidityEvent
    #[test]
    fn test_zero_copy_liquidity_event_new() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let base = ZeroCopyEvent::new_borrowed(metadata, data);
        let liquidity_event = ZeroCopyLiquidityEvent::new(base);

        assert_eq!(liquidity_event.base.raw_data(), data);
        // These fields are private, but we can test the public methods which should return None
        // since the base parse methods are stubs that return None
        let mut liquidity_event_mut = liquidity_event.clone();
        assert!(liquidity_event_mut.pool_address().is_none());
        assert!(liquidity_event_mut.lp_amount().is_none());
        assert!(liquidity_event_mut.token_a_amount().is_none());
        assert!(liquidity_event_mut.token_b_amount().is_none());
    }

    #[test]
    fn test_zero_copy_liquidity_event_lazy_parsing() {
        let data = b"test data";
        let metadata = create_test_metadata();
        let base = ZeroCopyEvent::new_borrowed(metadata, data);
        let mut liquidity_event = ZeroCopyLiquidityEvent::new(base);

        // Test that calling getter methods returns None (since parsing methods return None)
        assert!(liquidity_event.pool_address().is_none());
        assert!(liquidity_event.lp_amount().is_none());
        assert!(liquidity_event.token_a_amount().is_none());
        assert!(liquidity_event.token_b_amount().is_none());

        // Test that calling again uses cached values
        assert!(liquidity_event.pool_address().is_none());
        assert!(liquidity_event.lp_amount().is_none());
        assert!(liquidity_event.token_a_amount().is_none());
        assert!(liquidity_event.token_b_amount().is_none());
    }

    // Tests for ParseError
    #[test]
    fn test_parse_error_invalid_instruction_data() {
        let error = ParseError::InvalidInstructionData("test error".to_string());
        assert!(error
            .to_string()
            .contains("Invalid instruction data: test error"));
    }

    #[test]
    fn test_parse_error_insufficient_data() {
        let error = ParseError::InsufficientData {
            expected: 10,
            actual: 5,
        };
        assert!(error
            .to_string()
            .contains("Insufficient data length: expected 10, got 5"));
    }

    #[test]
    fn test_parse_error_unknown_discriminator() {
        let error = ParseError::UnknownDiscriminator {
            discriminator: vec![0x01, 0x02],
        };
        assert!(error.to_string().contains("Unknown discriminator"));
    }

    #[test]
    fn test_parse_error_memory_map_error() {
        let error = ParseError::MemoryMapError("mmap failed".to_string());
        assert!(error.to_string().contains("Memory map error: mmap failed"));
    }

    // Tests for MemoryMappedParser
    #[test]
    fn test_memory_mapped_parser_from_file_nonexistent() {
        let result = MemoryMappedParser::from_file("/nonexistent/file.dat");
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_mapped_parser_from_file_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data for mmap").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        assert_eq!(parser.size(), 18); // "test data for mmap" is 18 bytes
    }

    #[test]
    fn test_memory_mapped_parser_add_parser() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let mut parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: true,
        });

        parser.add_parser(mock_parser);
        // parsers field is private, so we can't directly access it
        // but we can verify indirectly by checking that the parser was added successfully
        let _events = parser.parse_all().unwrap();
        // If no error occurred, the parser was likely added successfully
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_no_parsers() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let events = parser.parse_all().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_with_parser() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let mut parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: true,
        });

        parser.add_parser(mock_parser);
        let events = parser.parse_all().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_parser_cannot_parse() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let mut parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: false,
        });

        parser.add_parser(mock_parser);
        let events = parser.parse_all().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_valid() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let slice = parser.data_slice(0, 4).unwrap();
        assert_eq!(slice, b"test");
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_invalid() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let slice = parser.data_slice(0, 10); // More than file size
        assert!(slice.is_none());
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_offset_out_of_bounds() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        let slice = parser.data_slice(10, 1); // Offset beyond file size
        assert!(slice.is_none());
    }

    #[test]
    fn test_memory_mapped_parser_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();

        let parser = MemoryMappedParser::from_file(file_path.to_str().unwrap()).unwrap();
        assert_eq!(parser.size(), 9);
    }

    // Tests for SIMDPatternMatcher
    #[test]
    fn test_simd_pattern_matcher_default() {
        let matcher = SIMDPatternMatcher::default();
        // patterns and protocols fields are private
        // We can test behavior indirectly through the public methods
        let test_data = vec![0x01, 0x02, 0x03];
        assert!(matcher.find_matches(&test_data).is_empty());
        assert_eq!(matcher.match_discriminator(&test_data), None);
    }

    #[test]
    fn test_simd_pattern_matcher_add_pattern() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        // patterns and protocols fields are private
        // We can test behavior indirectly by checking that the pattern works
        let test_data = vec![0x01, 0x02, 0x03];
        let matches = matcher.find_matches(&test_data);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (0, ProtocolType::Jupiter));
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_empty_data() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let matches = matcher.find_matches(&[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_no_matches() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let data = vec![0x03, 0x04, 0x05];
        let matches = matcher.find_matches(&data);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_single_match() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let data = vec![0x00, 0x01, 0x02, 0x03];
        let matches = matcher.find_matches(&data);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (1, ProtocolType::Jupiter));
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_multiple_matches() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01], ProtocolType::Jupiter);

        let data = vec![0x01, 0x02, 0x01, 0x03];
        let matches = matcher.find_matches(&data);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, ProtocolType::Jupiter));
        assert_eq!(matches[1], (2, ProtocolType::Jupiter));
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_overlapping_patterns() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);
        matcher.add_pattern(vec![0x02, 0x03], ProtocolType::RaydiumAmmV4);

        let data = vec![0x01, 0x02, 0x03];
        let matches = matcher.find_matches(&data);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches_insufficient_data() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02, 0x03], ProtocolType::Jupiter);

        let data = vec![0x01, 0x02]; // Only 2 bytes, pattern needs 3
        let matches = matcher.find_matches(&data);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_simd_pattern_matcher_match_discriminator_match() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let data = vec![0x01, 0x02, 0x03, 0x04];
        let result = matcher.match_discriminator(&data);
        assert_eq!(result, Some(ProtocolType::Jupiter));
    }

    #[test]
    fn test_simd_pattern_matcher_match_discriminator_no_match() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let data = vec![0x03, 0x04, 0x05, 0x06];
        let result = matcher.match_discriminator(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_simd_pattern_matcher_match_discriminator_insufficient_data() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);

        let data = vec![0x01]; // Only 1 byte, pattern needs 2
        let result = matcher.match_discriminator(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_simd_pattern_matcher_match_discriminator_multiple_patterns() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::Jupiter);
        matcher.add_pattern(vec![0x03, 0x04], ProtocolType::RaydiumAmmV4);

        let data = vec![0x03, 0x04, 0x05, 0x06];
        let result = matcher.match_discriminator(&data);
        assert_eq!(result, Some(ProtocolType::RaydiumAmmV4));
    }

    // Tests for CustomDeserializer
    #[test]
    fn test_custom_deserializer_new() {
        let data = vec![0x01, 0x02, 0x03];
        let deserializer = CustomDeserializer::new(&data);

        assert_eq!(deserializer.position(), 0);
        assert_eq!(deserializer.remaining(), 3);
    }

    #[test]
    fn test_custom_deserializer_read_u8_success() {
        let data = vec![0x42, 0x43, 0x44];
        let mut deserializer = CustomDeserializer::new(&data);

        let value = deserializer.read_u8().unwrap();
        assert_eq!(value, 0x42);
        assert_eq!(deserializer.position(), 1);
        assert_eq!(deserializer.remaining(), 2);
    }

    #[test]
    fn test_custom_deserializer_read_u8_insufficient_data() {
        let data = vec![];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u8();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_u32_le_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        let value = deserializer.read_u32_le().unwrap();
        assert_eq!(value, 0x04030201); // Little endian
        assert_eq!(deserializer.position(), 4);
        assert_eq!(deserializer.remaining(), 1);
    }

    #[test]
    fn test_custom_deserializer_read_u32_le_insufficient_data() {
        let data = vec![0x01, 0x02]; // Only 2 bytes, need 4
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u32_le();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 4);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_u64_le_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut deserializer = CustomDeserializer::new(&data);

        let value = deserializer.read_u64_le().unwrap();
        assert_eq!(value, 0x0807060504030201); // Little endian
        assert_eq!(deserializer.position(), 8);
        assert_eq!(deserializer.remaining(), 1);
    }

    #[test]
    fn test_custom_deserializer_read_u64_le_insufficient_data() {
        let data = vec![0x01, 0x02, 0x03]; // Only 3 bytes, need 8
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u64_le();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_pubkey_success() {
        let mut data = vec![0; 32];
        data[0] = 0x01; // Make it non-zero to be a valid pubkey
        let mut deserializer = CustomDeserializer::new(&data);

        let pubkey = deserializer.read_pubkey().unwrap();
        assert_eq!(deserializer.position(), 32);
        assert_eq!(deserializer.remaining(), 0);
        // Verify it's a valid pubkey by checking it can be converted back
        assert_eq!(pubkey.to_bytes()[0], 0x01);
    }

    #[test]
    fn test_custom_deserializer_read_pubkey_insufficient_data() {
        let data = vec![0x01, 0x02]; // Only 2 bytes, need 32
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_pubkey();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 32);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_bytes_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        let bytes = deserializer.read_bytes(3).unwrap();
        assert_eq!(bytes, &[0x01, 0x02, 0x03]);
        assert_eq!(deserializer.position(), 3);
        assert_eq!(deserializer.remaining(), 2);
    }

    #[test]
    fn test_custom_deserializer_read_bytes_insufficient_data() {
        let data = vec![0x01, 0x02];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_bytes(5);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 5);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_skip_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        deserializer.skip(3).unwrap();
        assert_eq!(deserializer.position(), 3);
        assert_eq!(deserializer.remaining(), 2);
    }

    #[test]
    fn test_custom_deserializer_skip_insufficient_data() {
        let data = vec![0x01, 0x02];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.skip(5);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 5);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_remaining_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        deserializer.skip(2).unwrap();
        let remaining = deserializer.remaining_data();
        assert_eq!(remaining, &[0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_custom_deserializer_remaining_data_empty() {
        let data = vec![0x01, 0x02];
        let mut deserializer = CustomDeserializer::new(&data);

        deserializer.skip(2).unwrap();
        let remaining = deserializer.remaining_data();
        assert!(remaining.is_empty());
    }

    // Tests for BatchEventParser
    #[test]
    fn test_batch_event_parser_new() {
        let parser = BatchEventParser::new(100);
        let stats = parser.get_stats();

        assert_eq!(stats.max_batch_size, 100);
        assert_eq!(stats.registered_parsers, 0);
        assert_eq!(stats.pattern_count, 0);
    }

    #[test]
    fn test_batch_event_parser_add_parser_raydium() {
        let mut parser = BatchEventParser::new(100);
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::RaydiumAmmV4,
            can_parse_result: true,
        });

        parser.add_parser(mock_parser);
        let stats = parser.get_stats();

        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_jupiter() {
        let mut parser = BatchEventParser::new(100);
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: true,
        });

        parser.add_parser(mock_parser);
        let stats = parser.get_stats();

        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_other() {
        let mut parser = BatchEventParser::new(100);
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::Other("Custom".to_string()),
            can_parse_result: true,
        });

        parser.add_parser(mock_parser);
        let stats = parser.get_stats();

        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_empty() {
        let parser = BatchEventParser::new(100);
        let batch: &[&[u8]] = &[];
        let metadatas = vec![];

        let result = parser.parse_batch(batch, metadatas).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_batch_event_parser_parse_batch_exceeds_max_size() {
        let parser = BatchEventParser::new(1); // Very small max size
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        let batch = &[data1.as_slice(), data2.as_slice()];
        let metadatas = vec![create_test_metadata(), create_test_metadata()];

        let result = parser.parse_batch(batch, metadatas);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidInstructionData(msg) => {
                assert!(msg.contains("Batch size 2 exceeds maximum 1"));
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    #[test]
    fn test_batch_event_parser_parse_batch_no_matching_parser() {
        let parser = BatchEventParser::new(100);
        let data = b"test data";
        let batch = &[data.as_slice()];
        let metadatas = vec![create_test_metadata()];

        let result = parser.parse_batch(batch, metadatas).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_batch_event_parser_parse_batch_with_matching_parser() {
        let mut parser = BatchEventParser::new(100);
        let mock_parser = Arc::new(MockParser {
            protocol: ProtocolType::RaydiumAmmV4,
            can_parse_result: true,
        });
        parser.add_parser(mock_parser);

        let data = vec![0x09, 0x01, 0x02]; // Starts with RaydiumAmmV4 discriminator
        let batch = &[data.as_slice()];
        let metadatas = vec![create_test_metadata()];

        let result = parser.parse_batch(batch, metadatas).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_parser_error() {
        #[derive(Debug)]
        struct ErrorParser;
        impl ByteSliceEventParser for ErrorParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: EventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Err(ParseError::InvalidInstructionData(
                    "Parser error".to_string(),
                ))
            }
            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }
            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::RaydiumAmmV4
            }
        }

        let mut parser = BatchEventParser::new(100);
        parser.add_parser(Arc::new(ErrorParser));

        let data = vec![0x09, 0x01, 0x02];
        let batch = &[data.as_slice()];
        let metadatas = vec![create_test_metadata()];

        let result = parser.parse_batch(batch, metadatas);
        assert!(result.is_err());
    }

    // Tests for BatchParserStats
    #[test]
    fn test_batch_parser_stats_clone() {
        let stats = BatchParserStats {
            registered_parsers: 5,
            max_batch_size: 100,
            pattern_count: 3,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.registered_parsers, 5);
        assert_eq!(cloned.max_batch_size, 100);
        assert_eq!(cloned.pattern_count, 3);
    }

    // Tests for RpcConnectionPool
    #[test]
    fn test_rpc_connection_pool_new() {
        let urls = vec!["http://localhost:8899".to_string()];
        let pool = RpcConnectionPool::new(urls);

        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_rpc_connection_pool_new_empty() {
        let urls = vec![];
        let pool = RpcConnectionPool::new(urls);

        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_rpc_connection_pool_new_multiple() {
        let urls = vec![
            "http://localhost:8899".to_string(),
            "http://localhost:8900".to_string(),
            "http://localhost:8901".to_string(),
        ];
        let pool = RpcConnectionPool::new(urls);

        assert_eq!(pool.size(), 3);
    }

    #[test]
    #[should_panic(expected = "No RPC clients in pool")]
    fn test_rpc_connection_pool_get_client_empty_pool() {
        let pool = RpcConnectionPool::new(vec![]);
        pool.get_client();
    }

    #[test]
    fn test_rpc_connection_pool_get_client_single() {
        let urls = vec!["http://localhost:8899".to_string()];
        let pool = RpcConnectionPool::new(urls);

        let client1 = pool.get_client();
        let client2 = pool.get_client();

        // Both should be the same client (round-robin with 1 client)
        assert!(Arc::ptr_eq(&client1, &client2));
    }

    #[test]
    fn test_rpc_connection_pool_get_client_round_robin() {
        let urls = vec![
            "http://localhost:8899".to_string(),
            "http://localhost:8900".to_string(),
        ];
        let pool = RpcConnectionPool::new(urls);

        let client1 = pool.get_client();
        let client2 = pool.get_client();
        let client3 = pool.get_client();

        // client1 and client3 should be the same (round-robin)
        assert!(Arc::ptr_eq(&client1, &client3));
        // client1 and client2 should be different
        assert!(!Arc::ptr_eq(&client1, &client2));
    }

    // Additional edge case tests
    #[test]
    fn test_mock_parser_parse_from_slice_empty_data() {
        let parser = MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: true,
        };

        let metadata = create_test_metadata();
        let result = parser.parse_from_slice(&[], metadata);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidInstructionData(msg) => {
                assert_eq!(msg, "Empty data");
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    #[test]
    fn test_mock_parser_can_parse() {
        let parser = MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: false,
        };

        assert!(!parser.can_parse(b"any data"));

        let parser = MockParser {
            protocol: ProtocolType::Jupiter,
            can_parse_result: true,
        };

        assert!(parser.can_parse(b"any data"));
    }

    #[test]
    fn test_mock_parser_protocol_type() {
        let parser = MockParser {
            protocol: ProtocolType::RaydiumAmmV4,
            can_parse_result: true,
        };

        assert_eq!(parser.protocol_type(), ProtocolType::RaydiumAmmV4);
    }

    // Test From trait implementation for ParseError
    #[test]
    fn test_parse_error_from_borsh_error() {
        let borsh_error =
            borsh::io::Error::new(borsh::io::ErrorKind::UnexpectedEof, "test borsh error");
        let parse_error: ParseError = borsh_error.into();

        match parse_error {
            ParseError::DeserializationError(_) => {} // Expected
            _ => panic!("Expected DeserializationError"),
        }
    }

    // Test error conversion for file operations
    #[test]
    fn test_memory_mapped_parser_from_file_io_error() {
        // Test with a directory instead of a file to trigger an IO error
        let dir = tempdir().unwrap();
        let result = MemoryMappedParser::from_file(dir.path().to_str().unwrap());
        assert!(result.is_err());
    }

    // Test remaining coverage for CustomDeserializer position tracking
    #[test]
    fn test_custom_deserializer_position_tracking() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut deserializer = CustomDeserializer::new(&data);

        assert_eq!(deserializer.position(), 0);
        assert_eq!(deserializer.remaining(), 9);

        let _ = deserializer.read_u8().unwrap();
        assert_eq!(deserializer.position(), 1);
        assert_eq!(deserializer.remaining(), 8);

        let _ = deserializer.read_u32_le().unwrap();
        assert_eq!(deserializer.position(), 5);
        assert_eq!(deserializer.remaining(), 4);

        let _ = deserializer.read_bytes(2).unwrap();
        assert_eq!(deserializer.position(), 7);
        assert_eq!(deserializer.remaining(), 2);

        deserializer.skip(1).unwrap();
        assert_eq!(deserializer.position(), 8);
        assert_eq!(deserializer.remaining(), 1);
    }
}
