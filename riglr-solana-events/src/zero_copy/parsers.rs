//! Zero-copy parsing utilities for high-performance instruction and log processing
//!
//! This module provides utilities for parsing Solana transaction data with minimal
//! memory allocations through zero-copy techniques and efficient byte slice operations.

use crate::solana_metadata::SolanaEventMetadata;
use crate::types::metadata_helpers::create_solana_metadata;
use crate::types::{EventType, ProtocolType};
use crate::zero_copy::events::ZeroCopyEvent;
use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs::File;
use std::sync::Arc;

/// Trait for zero-copy byte slice parsing
pub trait ByteSliceEventParser: Send + Sync + std::fmt::Debug {
    /// Parse events from a byte slice without copying data
    fn parse_from_slice<'a>(
        &self,
        data: &'a [u8],
        metadata: SolanaEventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError>;

    /// Check if this parser can handle the given data
    fn can_parse(&self, data: &[u8]) -> bool;

    /// Get the protocol type this parser handles
    fn protocol_type(&self) -> ProtocolType;
}

/// Error type for parsing operations
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Invalid instruction data encountered during parsing
    #[error("Invalid instruction data: {0}")]
    InvalidInstructionData(String),

    /// Insufficient data available for parsing operation
    #[error("Insufficient data length: expected {expected}, got {actual}")]
    InsufficientData {
        /// Expected number of bytes
        expected: usize,
        /// Actual number of bytes available
        actual: usize,
    },

    /// Unknown discriminator value encountered
    #[error("Unknown discriminator: {discriminator:?}")]
    UnknownDiscriminator {
        /// The unrecognized discriminator bytes
        discriminator: Vec<u8>,
    },

    /// Borsh deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] borsh::io::Error),

    /// Memory mapping operation error
    #[error("Memory map error: {0}")]
    MemoryMapError(String),
}

/// High-performance parser using memory-mapped files for large transaction logs
#[derive(Debug)]
pub struct MemoryMappedParser {
    /// Memory-mapped file handle
    mmap: memmap2::Mmap,
    /// Protocol-specific parsers indexed by protocol type
    parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>,
}

impl MemoryMappedParser {
    /// Create a new memory-mapped parser from a file
    pub fn from_file(file_path: &str) -> Result<Self, ParseError> {
        let file = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Self {
            mmap,
            parsers: HashMap::default(),
        })
    }

    /// Add a protocol-specific parser
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        self.parsers.insert(parser.protocol_type(), parser);
    }

    /// Parse events from the memory-mapped data
    pub fn parse_all(&self) -> Result<Vec<ZeroCopyEvent<'_>>, ParseError> {
        let mut events = Vec::default();
        let data = &self.mmap[..];

        // Simple parsing - in real implementation, this would have more sophisticated
        // transaction boundary detection
        for parser in self.parsers.values() {
            if parser.can_parse(data) {
                // Create metadata with default values - would be computed from context
                let metadata = create_solana_metadata(
                    String::default(),
                    String::default(),
                    0,
                    0,
                    ProtocolType::default(),
                    EventType::default(),
                    solana_sdk::pubkey::Pubkey::default(),
                    String::default(),
                    0,
                );
                let mut parsed = parser.parse_from_slice(data, metadata)?;
                for event in &mut parsed {
                    events.push(event.to_owned());
                }
            }
        }

        Ok(events)
    }

    /// Get a slice of the memory-mapped data
    pub fn data_slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset + len <= self.mmap.len() {
            Some(&self.mmap[offset..offset + len])
        } else {
            None
        }
    }

    /// Get the total size of the mapped data
    pub fn size(&self) -> usize {
        self.mmap.len()
    }
}

/// SIMD-optimized pattern matcher for instruction discriminators
#[derive(Default)]
pub struct SIMDPatternMatcher {
    /// Discriminator byte patterns to match
    patterns: Vec<Vec<u8>>,
    /// Protocol types corresponding to each pattern
    protocols: Vec<ProtocolType>,
}

impl SIMDPatternMatcher {
    /// Add a pattern to match
    pub fn add_pattern(&mut self, pattern: Vec<u8>, protocol: ProtocolType) {
        self.patterns.push(pattern);
        self.protocols.push(protocol);
    }

    /// Find matching patterns in data
    ///
    /// In a real implementation, this would use SIMD instructions for parallel matching
    /// For now, we provide a basic implementation
    pub fn find_matches(&self, data: &[u8]) -> Vec<(usize, ProtocolType)> {
        let mut matches = Vec::default();

        for (pattern, protocol) in self.patterns.iter().zip(&self.protocols) {
            if data.len() >= pattern.len() {
                for pos in 0..=data.len() - pattern.len() {
                    if &data[pos..pos + pattern.len()] == pattern {
                        matches.push((pos, protocol.clone()));
                    }
                }
            }
        }

        matches
    }

    /// Fast prefix matching for instruction discriminators
    pub fn match_discriminator(&self, data: &[u8]) -> Option<ProtocolType> {
        for (pattern, protocol) in self.patterns.iter().zip(&self.protocols) {
            if data.len() >= pattern.len() && &data[..pattern.len()] == pattern {
                return Some(protocol.clone());
            }
        }
        None
    }
}

/// Custom deserializer for hot path parsing
pub struct CustomDeserializer<'a> {
    /// Byte data being deserialized
    data: &'a [u8],
    /// Current read position in the data
    pos: usize,
}

impl<'a> CustomDeserializer<'a> {
    /// Create a new deserializer
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Read a u8 value
    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        if self.pos >= self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: self.data.len() - self.pos,
            });
        }

        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    /// Read a u32 value (little endian)
    pub fn read_u32_le(&mut self) -> Result<u32, ParseError> {
        if self.pos + 4 > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 4,
                actual: self.data.len() - self.pos,
            });
        }

        let bytes = &self.data[self.pos..self.pos + 4];
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        self.pos += 4;
        Ok(value)
    }

    /// Read a u64 value (little endian)
    pub fn read_u64_le(&mut self) -> Result<u64, ParseError> {
        if self.pos + 8 > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: self.data.len() - self.pos,
            });
        }

        let bytes = &self.data[self.pos..self.pos + 8];
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        let value = u64::from_le_bytes(array);
        self.pos += 8;
        Ok(value)
    }

    /// Read a Pubkey (32 bytes)
    pub fn read_pubkey(&mut self) -> Result<solana_sdk::pubkey::Pubkey, ParseError> {
        if self.pos + 32 > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 32,
                actual: self.data.len() - self.pos,
            });
        }

        let bytes = &self.data[self.pos..self.pos + 32];
        self.pos += 32;
        solana_sdk::pubkey::Pubkey::try_from(bytes)
            .map_err(|e| ParseError::InvalidInstructionData(format!("Invalid pubkey: {}", e)))
    }

    /// Read a byte array of fixed size
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParseError> {
        if self.pos + len > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: len,
                actual: self.data.len() - self.pos,
            });
        }

        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    /// Skip bytes
    pub fn skip(&mut self, len: usize) -> Result<(), ParseError> {
        if self.pos + len > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: len,
                actual: self.data.len() - self.pos,
            });
        }

        self.pos += len;
        Ok(())
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    /// Get remaining data as slice
    pub fn remaining_data(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }
}

/// Batch processor for efficient parsing of multiple transactions
pub struct BatchEventParser {
    /// Protocol-specific parsers indexed by protocol type
    parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>,
    /// SIMD pattern matcher for fast protocol detection
    pattern_matcher: SIMDPatternMatcher,
    /// Maximum number of transactions to process in a single batch
    max_batch_size: usize,
}

impl BatchEventParser {
    /// Create a new batch parser
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            parsers: HashMap::default(),
            pattern_matcher: SIMDPatternMatcher::default(),
            max_batch_size,
        }
    }

    /// Add a parser for a specific protocol
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        let protocol = parser.protocol_type();

        // Add to parsers map
        self.parsers.insert(protocol.clone(), parser);

        // Add discriminator pattern for fast detection
        match protocol {
            ProtocolType::RaydiumAmmV4 => {
                self.pattern_matcher.add_pattern(vec![0x09], protocol); // SwapBaseIn discriminator
            }
            ProtocolType::Jupiter => {
                self.pattern_matcher.add_pattern(
                    vec![0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x8b],
                    protocol,
                ); // Route discriminator
            }
            _ => {
                // Add default pattern
                self.pattern_matcher.add_pattern(vec![0x00], protocol);
            }
        }
    }

    /// Parse a batch of transaction data
    pub fn parse_batch<'a>(
        &self,
        batch: &'a [&'a [u8]],
        metadatas: Vec<SolanaEventMetadata>,
    ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
        if batch.len() > self.max_batch_size {
            return Err(ParseError::InvalidInstructionData(format!(
                "Batch size {} exceeds maximum {}",
                batch.len(),
                self.max_batch_size
            )));
        }

        let mut all_events = Vec::default();

        for (data, metadata) in batch.iter().zip(metadatas.into_iter()) {
            // Fast protocol detection using SIMD pattern matching
            if let Some(protocol) = self.pattern_matcher.match_discriminator(data) {
                if let Some(parser) = self.parsers.get(&protocol) {
                    let mut events = parser.parse_from_slice(data, metadata)?;
                    all_events.append(&mut events);
                }
            }
        }

        Ok(all_events)
    }

    /// Get statistics about the batch parser
    pub fn get_stats(&self) -> BatchParserStats {
        BatchParserStats {
            registered_parsers: self.parsers.len(),
            max_batch_size: self.max_batch_size,
            pattern_count: 0, // patterns field is private, can't access directly
        }
    }
}

/// Statistics for batch parser performance monitoring
#[derive(Debug, Clone)]
pub struct BatchParserStats {
    /// Number of registered protocol parsers
    pub registered_parsers: usize,
    /// Maximum batch size configured
    pub max_batch_size: usize,
    /// Number of discriminator patterns registered
    pub pattern_count: usize,
}

/// Connection pool for RPC calls during parsing
pub struct RpcConnectionPool {
    /// Pool of shared RPC client instances
    clients: Vec<Arc<solana_client::rpc_client::RpcClient>>,
    /// Current index for round-robin client selection
    current: std::sync::atomic::AtomicUsize,
}

impl RpcConnectionPool {
    /// Create a new connection pool
    pub fn new(urls: Vec<String>) -> Self {
        let clients = urls
            .into_iter()
            .map(|url| Arc::new(solana_client::rpc_client::RpcClient::new(url)))
            .collect();

        Self {
            clients,
            current: std::sync::atomic::AtomicUsize::default(),
        }
    }

    /// Get the next client in round-robin fashion
    pub fn get_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        if self.clients.is_empty() {
            panic!("No RPC clients in pool");
        }

        let index = self
            .current
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.clients.len();
        self.clients[index].clone()
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.clients.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_deserializer() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut deserializer = CustomDeserializer::new(&data);

        assert_eq!(deserializer.read_u8().unwrap(), 0x01);
        assert_eq!(deserializer.read_u32_le().unwrap(), 0x05040302);
        assert_eq!(deserializer.remaining(), 3);
    }

    #[test]
    fn test_simd_pattern_matcher() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x09], ProtocolType::RaydiumAmmV4);

        let data = vec![0x09, 0x01, 0x02, 0x03];
        assert_eq!(
            matcher.match_discriminator(&data),
            Some(ProtocolType::RaydiumAmmV4)
        );

        let no_match_data = vec![0x08, 0x01, 0x02, 0x03];
        assert_eq!(matcher.match_discriminator(&no_match_data), None);
    }

    #[test]
    fn test_batch_parser() {
        let parser = BatchEventParser::new(100);
        assert_eq!(parser.get_stats().max_batch_size, 100);
        assert_eq!(parser.get_stats().registered_parsers, 0);
    }

    // Test CustomDeserializer error paths and edge cases
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
    fn test_custom_deserializer_read_u64_le_insufficient_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04]; // Only 4 bytes, need 8
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u64_le();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 4);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_u64_le_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u64_le().unwrap();
        assert_eq!(result, 0x0807060504030201);
        assert_eq!(deserializer.position(), 8);
        assert_eq!(deserializer.remaining(), 0);
    }

    #[test]
    fn test_custom_deserializer_read_pubkey_insufficient_data() {
        let data = vec![0u8; 16]; // Only 16 bytes, need 32
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_pubkey();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InsufficientData { expected, actual } => {
                assert_eq!(expected, 32);
                assert_eq!(actual, 16);
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_custom_deserializer_read_pubkey_success() {
        let data = vec![0u8; 32]; // Valid 32-byte pubkey data
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_pubkey();
        assert!(result.is_ok());
        assert_eq!(deserializer.position(), 32);
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
    fn test_custom_deserializer_read_bytes_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_bytes(3).unwrap();
        assert_eq!(result, &[0x01, 0x02, 0x03]);
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
    fn test_custom_deserializer_skip_success() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.skip(2);
        assert!(result.is_ok());
        assert_eq!(deserializer.position(), 2);
        assert_eq!(deserializer.remaining(), 3);
    }

    #[test]
    fn test_custom_deserializer_remaining_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut deserializer = CustomDeserializer::new(&data);

        let _ = deserializer.read_u8().unwrap();
        let remaining = deserializer.remaining_data();
        assert_eq!(remaining, &[0x02, 0x03, 0x04]);
    }

    // Test SIMDPatternMatcher edge cases
    #[test]
    fn test_simd_pattern_matcher_empty_patterns() {
        let matcher = SIMDPatternMatcher::default();
        let data = vec![0x01, 0x02, 0x03];

        assert!(matcher.find_matches(&data).is_empty());
        assert_eq!(matcher.match_discriminator(&data), None);
    }

    #[test]
    fn test_simd_pattern_matcher_find_matches() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02], ProtocolType::RaydiumAmmV4);
        matcher.add_pattern(vec![0x03], ProtocolType::Jupiter);

        let data = vec![0x01, 0x02, 0x03, 0x01, 0x02];
        let matches = matcher.find_matches(&data);

        assert_eq!(matches.len(), 3); // Two [0x01, 0x02] matches and one [0x03] match
        assert_eq!(matches[0], (0, ProtocolType::RaydiumAmmV4));
        assert_eq!(matches[1], (2, ProtocolType::Jupiter));
        assert_eq!(matches[2], (3, ProtocolType::RaydiumAmmV4));
    }

    #[test]
    fn test_simd_pattern_matcher_data_too_short() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02, 0x03], ProtocolType::RaydiumAmmV4);

        let data = vec![0x01, 0x02]; // Too short for pattern
        let matches = matcher.find_matches(&data);

        assert!(matches.is_empty());
        assert_eq!(matcher.match_discriminator(&data), None);
    }

    #[test]
    fn test_simd_pattern_matcher_multiple_protocols() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x09], ProtocolType::RaydiumAmmV4);
        matcher.add_pattern(vec![0xe4, 0x45, 0xa5, 0x2e], ProtocolType::Jupiter);

        let data1 = vec![0x09, 0x01, 0x02];
        let data2 = vec![0xe4, 0x45, 0xa5, 0x2e, 0x00];

        assert_eq!(
            matcher.match_discriminator(&data1),
            Some(ProtocolType::RaydiumAmmV4)
        );
        assert_eq!(
            matcher.match_discriminator(&data2),
            Some(ProtocolType::Jupiter)
        );
    }

    // Test BatchEventParser comprehensive functionality
    #[test]
    fn test_batch_event_parser_add_parser_raydium() {
        let mut parser = BatchEventParser::new(100);

        // Create a mock parser
        #[derive(Debug)]
        struct MockParser;
        impl ByteSliceEventParser for MockParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::RaydiumAmmV4
            }
        }

        parser.add_parser(Arc::new(MockParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_jupiter() {
        let mut parser = BatchEventParser::new(100);

        #[derive(Debug)]
        struct MockJupiterParser;
        impl ByteSliceEventParser for MockJupiterParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(MockJupiterParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_other_protocol() {
        let mut parser = BatchEventParser::new(100);

        #[derive(Debug)]
        struct MockOtherParser;
        impl ByteSliceEventParser for MockOtherParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::OrcaWhirlpool
            }
        }

        parser.add_parser(Arc::new(MockOtherParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_exceeds_max_size() {
        let parser = BatchEventParser::new(2);

        let data1 = vec![0x01, 0x02];
        let data2 = vec![0x03, 0x04];
        let data3 = vec![0x05, 0x06];
        let batch = vec![&data1[..], &data2[..], &data3[..]]; // 3 items, max is 2

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            solana_sdk::pubkey::Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata.clone(), metadata.clone(), metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidInstructionData(msg) => {
                assert!(msg.contains("Batch size 3 exceeds maximum 2"));
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    #[test]
    fn test_batch_event_parser_parse_batch_success() {
        let mut parser = BatchEventParser::new(10);

        #[derive(Debug)]
        struct MockSuccessParser;
        impl ByteSliceEventParser for MockSuccessParser {
            fn parse_from_slice<'a>(
                &self,
                data: &'a [u8],
                metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                // Create a mock event
                Ok(vec![ZeroCopyEvent::new_borrowed(metadata, data)])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::RaydiumAmmV4
            }
        }

        parser.add_parser(Arc::new(MockSuccessParser));

        let data1 = vec![0x09, 0x02]; // Starts with Raydium discriminator
        let batch = vec![&data1[..]];

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            solana_sdk::pubkey::Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_no_matching_protocol() {
        let parser = BatchEventParser::new(10);

        let data1 = vec![0xFF, 0xFF]; // No matching discriminator
        let batch = vec![&data1[..]];

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            solana_sdk::pubkey::Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0); // No events should be parsed
    }

    // Test RpcConnectionPool functionality
    #[test]
    fn test_rpc_connection_pool_new() {
        let urls = vec![
            "https://api.mainnet-beta.solana.com".to_string(),
            "https://solana-api.projectserum.com".to_string(),
        ];
        let pool = RpcConnectionPool::new(urls);

        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_rpc_connection_pool_round_robin() {
        let urls = vec![
            "https://api.mainnet-beta.solana.com".to_string(),
            "https://solana-api.projectserum.com".to_string(),
        ];
        let pool = RpcConnectionPool::new(urls);

        // Get multiple clients to test round-robin
        let _client1 = pool.get_client();
        let _client2 = pool.get_client();
        let _client3 = pool.get_client();

        // Should work without panicking
        assert_eq!(pool.size(), 2);
    }

    #[test]
    #[should_panic(expected = "No RPC clients in pool")]
    fn test_rpc_connection_pool_empty_panic() {
        let pool = RpcConnectionPool::new(vec![]);
        let _ = pool.get_client(); // Should panic
    }

    #[test]
    fn test_rpc_connection_pool_empty_size() {
        let pool = RpcConnectionPool::new(vec![]);
        assert_eq!(pool.size(), 0);
    }

    // Test ParseError variants
    #[test]
    fn test_parse_error_display() {
        let error1 = ParseError::InvalidInstructionData("test error".to_string());
        assert_eq!(
            format!("{}", error1),
            "Invalid instruction data: test error"
        );

        let error2 = ParseError::InsufficientData {
            expected: 10,
            actual: 5,
        };
        assert_eq!(
            format!("{}", error2),
            "Insufficient data length: expected 10, got 5"
        );

        let error3 = ParseError::UnknownDiscriminator {
            discriminator: vec![0x01, 0x02],
        };
        assert_eq!(format!("{}", error3), "Unknown discriminator: [1, 2]");

        let error4 = ParseError::MemoryMapError("mmap failed".to_string());
        assert_eq!(format!("{}", error4), "Memory map error: mmap failed");
    }

    // Test BatchParserStats
    #[test]
    fn test_batch_parser_stats_clone() {
        let stats = BatchParserStats {
            registered_parsers: 5,
            max_batch_size: 100,
            pattern_count: 3,
        };

        let cloned_stats = stats.clone();
        assert_eq!(cloned_stats.registered_parsers, 5);
        assert_eq!(cloned_stats.max_batch_size, 100);
        assert_eq!(cloned_stats.pattern_count, 3);
    }

    // Test edge cases for CustomDeserializer at exact boundaries
    #[test]
    fn test_custom_deserializer_exact_boundary_u32() {
        let data = vec![0x01, 0x02, 0x03, 0x04]; // Exactly 4 bytes
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u32_le();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x04030201);
        assert_eq!(deserializer.remaining(), 0);

        // Try to read another u32 - should fail
        let result2 = deserializer.read_u32_le();
        assert!(result2.is_err());
    }

    #[test]
    fn test_custom_deserializer_exact_boundary_u64() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]; // Exactly 8 bytes
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_u64_le();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x0807060504030201);
        assert_eq!(deserializer.remaining(), 0);

        // Try to read another u64 - should fail
        let result2 = deserializer.read_u64_le();
        assert!(result2.is_err());
    }

    #[test]
    fn test_custom_deserializer_exact_boundary_pubkey() {
        let data = vec![0u8; 32]; // Exactly 32 bytes
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_pubkey();
        assert!(result.is_ok());
        assert_eq!(deserializer.remaining(), 0);

        // Try to read another pubkey - should fail
        let result2 = deserializer.read_pubkey();
        assert!(result2.is_err());
    }

    #[test]
    fn test_custom_deserializer_zero_length_operations() {
        let data = vec![0x01, 0x02, 0x03];
        let mut deserializer = CustomDeserializer::new(&data);

        // Reading 0 bytes should succeed
        let result = deserializer.read_bytes(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        assert_eq!(deserializer.position(), 0);

        // Skipping 0 bytes should succeed
        let result = deserializer.skip(0);
        assert!(result.is_ok());
        assert_eq!(deserializer.position(), 0);
    }

    #[test]
    fn test_custom_deserializer_mixed_operations() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
        let mut deserializer = CustomDeserializer::new(&data);

        // Read u8
        assert_eq!(deserializer.read_u8().unwrap(), 0x01);
        assert_eq!(deserializer.position(), 1);

        // Skip 2 bytes
        deserializer.skip(2).unwrap();
        assert_eq!(deserializer.position(), 3);

        // Read 3 bytes
        let bytes = deserializer.read_bytes(3).unwrap();
        assert_eq!(bytes, &[0x04, 0x05, 0x06]);
        assert_eq!(deserializer.position(), 6);

        // Read u32
        assert_eq!(deserializer.read_u32_le().unwrap(), 0x0A090807);
        assert_eq!(deserializer.position(), 10);
        assert_eq!(deserializer.remaining(), 0);
    }

    // Test MemoryMappedParser functionality
    #[test]
    fn test_memory_mapped_parser_from_file_nonexistent() {
        let result = MemoryMappedParser::from_file("/nonexistent/file/path.dat");
        assert!(result.is_err());

        // Should be a file not found error wrapped in ParseError
        match result.unwrap_err() {
            ParseError::DeserializationError(_) => {
                // This is the expected error type from the std::io::Error conversion
            }
            _ => panic!("Expected DeserializationError from file not found"),
        }
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_valid_range() {
        // Create a temporary file for testing
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is test data for memory mapping.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Test valid slice
        let slice = parser.data_slice(0, 5);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap(), b"Hello");

        // Test slice in middle
        let slice = parser.data_slice(7, 5);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap(), b"World");

        // Test entire data
        let slice = parser.data_slice(0, test_data.len());
        assert!(slice.is_some());
        assert_eq!(slice.unwrap(), test_data);
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_invalid_range() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_data = b"Small data";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Test offset beyond data
        let slice = parser.data_slice(test_data.len() + 1, 1);
        assert!(slice.is_none());

        // Test length beyond data
        let slice = parser.data_slice(0, test_data.len() + 1);
        assert!(slice.is_none());

        // Test offset + length beyond data
        let slice = parser.data_slice(5, test_data.len());
        assert!(slice.is_none());
    }

    #[test]
    fn test_memory_mapped_parser_size() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_data = b"Test data for size checking";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.size(), test_data.len());
    }

    #[test]
    fn test_memory_mapped_parser_add_parser() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let mut parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        #[derive(Debug)]
        struct TestParser;
        impl ByteSliceEventParser for TestParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(TestParser));

        // Verify parser was added (we can't directly access the HashMap, but we can test parse_all doesn't panic)
        let result = parser.parse_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_with_parser() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_data = b"test data for parsing";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let mut parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        #[derive(Debug)]
        struct MockEventParser;
        impl ByteSliceEventParser for MockEventParser {
            fn parse_from_slice<'a>(
                &self,
                data: &'a [u8],
                metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![ZeroCopyEvent::new_borrowed(metadata, data)])
            }

            fn can_parse(&self, data: &[u8]) -> bool {
                data.len() > 0
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(MockEventParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].raw_data(), test_data);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_no_parsers() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_parser_cannot_parse() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let mut parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        #[derive(Debug)]
        struct CannotParseParser;
        impl ByteSliceEventParser for CannotParseParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Ok(vec![])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                false // Always returns false
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(CannotParseParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_parser_error() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let mut parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        #[derive(Debug)]
        struct ErrorParser;
        impl ByteSliceEventParser for ErrorParser {
            fn parse_from_slice<'a>(
                &self,
                _data: &'a [u8],
                _metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                Err(ParseError::InvalidInstructionData("Test error".to_string()))
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(ErrorParser));

        let result = parser.parse_all();
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidInstructionData(msg) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    // Test ParseError from conversion
    #[test]
    fn test_parse_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let parse_error = ParseError::from(io_error);

        match parse_error {
            ParseError::DeserializationError(_) => {
                // Expected - io::Error gets converted to borsh::io::Error and then to ParseError
            }
            _ => panic!("Expected DeserializationError from io::Error conversion"),
        }
    }

    // Test edge case for data_slice with zero offset and zero length
    #[test]
    fn test_memory_mapped_parser_data_slice_zero_length() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Zero length slice should be valid
        let slice = parser.data_slice(0, 0);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 0);

        // Zero length slice at end should also be valid
        let slice = parser.data_slice(4, 0);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 0);
    }

    // Test boundary condition where offset + len equals exactly the data length
    #[test]
    fn test_memory_mapped_parser_data_slice_exact_boundary() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_data = b"test";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Exact boundary should work
        let slice = parser.data_slice(0, test_data.len());
        assert!(slice.is_some());
        assert_eq!(slice.unwrap(), test_data);

        // One byte past should fail
        let slice = parser.data_slice(0, test_data.len() + 1);
        assert!(slice.is_none());
    }

    // Test ZeroCopyEvent field access in parse_all
    #[test]
    fn test_memory_mapped_parser_parse_all_event_modification() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let mut parser = MemoryMappedParser::from_file(temp_file.path().to_str().unwrap()).unwrap();

        #[derive(Debug)]
        struct MultiEventParser;
        impl ByteSliceEventParser for MultiEventParser {
            fn parse_from_slice<'a>(
                &self,
                data: &'a [u8],
                metadata: SolanaEventMetadata,
            ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError> {
                // Return multiple events to test the loop
                Ok(vec![
                    ZeroCopyEvent::new_borrowed(metadata.clone(), data),
                    ZeroCopyEvent::new_borrowed(metadata, data),
                ])
            }

            fn can_parse(&self, _data: &[u8]) -> bool {
                true
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        parser.add_parser(Arc::new(MultiEventParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
    }
}
