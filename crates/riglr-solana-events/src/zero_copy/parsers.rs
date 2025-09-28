//! Zero-copy parsing utilities for high-performance instruction and log processing
//!
//! This module provides utilities for parsing Solana transaction data with minimal
//! memory allocations through zero-copy techniques and efficient byte slice operations.

extern crate alloc;

use crate::solana_metadata::SolanaEventMetadata;
use crate::types::metadata_helpers::create_solana_metadata;
use crate::types::{EventType, ProtocolType};
use crate::zero_copy::events::ZeroCopyEvent;
use alloc::sync::Arc;
use borsh::io::Error as BorshIoError;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::sync::atomic::{AtomicUsize, Ordering};
use memmap2::MmapOptions;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs::File;

/// Trait for zero-copy byte slice parsing
pub trait ByteSliceEventParser: Send + Sync + Debug {
    /// Check if this parser can handle the given data
    fn can_parse(&self, data: &[u8]) -> bool;

    /// Parse events from a byte slice without copying data
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the data cannot be parsed or is malformed.
    fn parse_from_slice<'data>(
        &self,
        data: &'data [u8],
        metadata: SolanaEventMetadata,
    ) -> Result<Vec<ZeroCopyEvent<'data>>, ParseError>;

    /// Get the protocol type this parser handles
    fn protocol_type(&self) -> ProtocolType;
}

/// Error type for parsing operations
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError {
    /// Borsh deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] BorshIoError),

    /// Insufficient data available for parsing operation
    #[error("Insufficient data length: expected {expected}, got {actual}")]
    InsufficientData {
        /// Expected number of bytes
        expected: usize,
        /// Actual number of bytes available
        actual: usize,
    },

    /// Invalid instruction data encountered during parsing
    #[error("Invalid instruction data: {0}")]
    InvalidInstructionData(String),

    /// Memory mapping operation error
    #[error("Memory map error: {0}")]
    MemoryMapError(String),

    /// No RPC clients available in connection pool
    #[error("No RPC clients available in connection pool")]
    NoRpcClientsAvailable,

    /// Unknown discriminator value encountered
    #[error("Unknown discriminator: {discriminator:?}")]
    UnknownDiscriminator {
        /// The unrecognized discriminator bytes
        discriminator: Vec<u8>,
    },
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
    /// Add a protocol-specific parser
    #[inline]
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        self.parsers.insert(parser.protocol_type(), parser);
    }

    /// Get a slice of the memory-mapped data
    ///
    /// # Panics
    ///
    /// Panics if the range `offset..offset + len` is somehow out of bounds despite
    /// the `saturating_add` bounds check. This should be impossible under normal operation.
    #[must_use]
    #[inline]
    pub fn data_slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        (offset.saturating_add(len) <= self.mmap.len()).then(|| {
            self.mmap
                .get(offset..offset.saturating_add(len))
                .unwrap_or(&[])
        })
    }

    /// Create a new memory-mapped parser from a file
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the file cannot be opened or memory-mapped.
    #[inline]
    #[expect(unsafe_code)]
    pub fn from_file(file_path: &str) -> Result<Self, ParseError> {
        let file = File::open(file_path)?;
        // SAFETY: Memory mapping a file is safe when the file handle is valid
        // and we don't modify the mapped memory. The Mmap type ensures proper cleanup.
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Self {
            mmap,
            parsers: HashMap::default(),
        })
    }

    /// Parse events from the memory-mapped data
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails for any protocol.
    #[inline]
    pub fn parse_all(&self) -> Result<Vec<ZeroCopyEvent<'_>>, ParseError> {
        let mut events = Vec::default();
        let data = &*self.mmap;

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
                    Pubkey::default(),
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

    /// Get the total size of the mapped data
    #[must_use]
    #[inline]
    pub fn size(&self) -> usize {
        self.mmap.len()
    }
}

/// SIMD-optimized pattern matcher for instruction discriminators
#[derive(Debug, Default)]
pub struct SIMDPatternMatcher {
    /// Discriminator byte patterns to match
    patterns: Vec<Vec<u8>>,
    /// Protocol types corresponding to each pattern
    protocols: Vec<ProtocolType>,
}

impl SIMDPatternMatcher {
    /// Add a pattern to match
    #[inline]
    pub fn add_pattern(&mut self, pattern: Vec<u8>, protocol: ProtocolType) {
        self.patterns.push(pattern);
        self.protocols.push(protocol);
    }

    /// Find matching patterns in data
    ///
    /// In a real implementation, this would use SIMD instructions for parallel matching
    /// For now, we provide a basic implementation
    ///
    /// # Panics
    ///
    /// Panics if the slice range is somehow out of bounds despite the loop condition bounds checking.
    /// This should be impossible under normal operation as the loop ensures `pos + pattern.len() <= data.len()`.
    #[must_use]
    #[inline]
    pub fn find_matches(&self, data: &[u8]) -> Vec<(usize, ProtocolType)> {
        let mut matches = Vec::default();

        for (pattern, protocol) in self.patterns.iter().zip(&self.protocols) {
            if data.len() >= pattern.len() {
                for pos in 0..=data.len().saturating_sub(pattern.len()) {
                    if data
                        .get(pos..pos.saturating_add(pattern.len()))
                        .is_some_and(|slice| slice == pattern)
                    {
                        matches.push((pos, protocol.clone()));
                    }
                }
            }
        }

        matches
    }

    /// Fast prefix matching for instruction discriminators
    ///
    /// # Panics
    ///
    /// Panics if the data slice cannot be obtained despite length bounds checking.
    /// This should be impossible under normal operation.
    #[must_use]
    #[inline]
    pub fn match_discriminator(&self, data: &[u8]) -> Option<ProtocolType> {
        for (pattern, protocol) in self.patterns.iter().zip(&self.protocols) {
            if data.len() >= pattern.len()
                && data
                    .get(..pattern.len())
                    .is_some_and(|slice| slice == pattern)
            {
                return Some(protocol.clone());
            }
        }
        None
    }
}

/// Custom deserializer for hot path parsing
#[derive(Debug)]
pub struct CustomDeserializer<'data> {
    /// Byte data being deserialized
    data: &'data [u8],
    /// Current read position in the data
    pos: usize,
}

impl<'data> CustomDeserializer<'data> {
    /// Create a new deserializer
    #[must_use]
    #[inline]
    pub const fn new(data: &'data [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Get current position
    #[must_use]
    #[inline]
    pub const fn position(&self) -> usize {
        self.pos
    }

    /// Read a byte array of fixed size
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientData` if there are not enough bytes.
    ///
    /// # Panics
    ///
    /// Panics if the slice range is somehow out of bounds despite length bounds checking.
    /// This should be impossible under normal operation as the function validates `self.pos + len <= self.data.len()` before slicing.
    #[inline]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'data [u8], ParseError> {
        if self.pos.saturating_add(len) > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: len,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        let end_pos = self.pos.saturating_add(len);
        let bytes =
            self.data
                .get(self.pos..end_pos)
                .ok_or_else(|| ParseError::InsufficientData {
                    expected: len,
                    actual: self.data.len().saturating_sub(self.pos),
                })?;
        self.pos = self.pos.saturating_add(len);
        Ok(bytes)
    }

    /// Read a Pubkey (32 bytes)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if there are insufficient bytes or invalid pubkey data.
    ///
    /// # Panics
    ///
    /// Panics if the slice range for 32 bytes is somehow out of bounds despite length bounds checking.
    /// This should be impossible under normal operation as the function validates `self.pos + 32 <= self.data.len()` before slicing.
    #[inline]
    pub fn read_pubkey(&mut self) -> Result<Pubkey, ParseError> {
        if self.pos.saturating_add(32) > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 32,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        let bytes = self
            .data
            .get(self.pos..self.pos.saturating_add(32))
            .ok_or_else(|| ParseError::InsufficientData {
                expected: 32,
                actual: self.data.len().saturating_sub(self.pos),
            })?;
        self.pos = self.pos.saturating_add(32);
        Pubkey::try_from(bytes)
            .map_err(|e| ParseError::InvalidInstructionData(format!("Invalid pubkey: {e}")))
    }

    /// Read a u32 value (little endian)
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientData` if there are not enough bytes.
    ///
    /// # Panics
    ///
    /// Panics if byte slice positions are out of bounds despite bounds checking.
    /// This should be impossible under normal operation.
    #[inline]
    pub fn read_u32_le(&mut self) -> Result<u32, ParseError> {
        if self.pos.saturating_add(4) > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 4,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        let bytes = self
            .data
            .get(self.pos..self.pos.saturating_add(4))
            .ok_or_else(|| ParseError::InsufficientData {
                expected: 4,
                actual: self.data.len().saturating_sub(self.pos),
            })?;

        let array: [u8; 4] = bytes.try_into().map_err(|_| ParseError::InsufficientData {
            expected: 4,
            actual: bytes.len(),
        })?;
        let value = u32::from_le_bytes(array);
        self.pos = self.pos.saturating_add(4);
        Ok(value)
    }

    /// Read a u64 value (little endian)
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientData` if there are not enough bytes.
    ///
    /// # Panics
    ///
    /// Panics if the slice range for 8 bytes is somehow out of bounds despite length bounds checking.
    /// This should be impossible under normal operation as the function validates `self.pos + 8 <= self.data.len()` before slicing.
    #[inline]
    pub fn read_u64_le(&mut self) -> Result<u64, ParseError> {
        if self.pos.saturating_add(8) > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 8,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        let bytes = self
            .data
            .get(self.pos..self.pos.saturating_add(8))
            .ok_or_else(|| ParseError::InsufficientData {
                expected: 8,
                actual: self.data.len().saturating_sub(self.pos),
            })?;
        let array: [u8; 8] = bytes.try_into().map_err(|_| ParseError::InsufficientData {
            expected: 8,
            actual: bytes.len(),
        })?;
        let value = u64::from_le_bytes(array);
        self.pos = self.pos.saturating_add(8);
        Ok(value)
    }

    /// Read a u8 value
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientData` if there are not enough bytes.
    ///
    /// # Panics
    ///
    /// Panics if the position is out of bounds despite bounds checking.
    /// This should be impossible under normal operation.
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        if self.pos >= self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: 1,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        let value = self
            .data
            .get(self.pos)
            .ok_or_else(|| ParseError::InsufficientData {
                expected: 1,
                actual: self.data.len().saturating_sub(self.pos),
            })?
            .to_owned();
        self.pos = self.pos.saturating_add(1);
        Ok(value)
    }

    /// Get remaining bytes
    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Get remaining data as slice
    ///
    /// Returns an empty slice if position is at or beyond the end of data.
    #[must_use]
    #[inline]
    pub fn remaining_data(&self) -> &'data [u8] {
        self.data.get(self.pos..).unwrap_or(&[])
    }

    /// Skip bytes
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientData` if there are not enough bytes to skip.
    #[inline]
    pub const fn skip(&mut self, len: usize) -> Result<(), ParseError> {
        if self.pos.saturating_add(len) > self.data.len() {
            return Err(ParseError::InsufficientData {
                expected: len,
                actual: self.data.len().saturating_sub(self.pos),
            });
        }

        self.pos = self.pos.saturating_add(len);
        Ok(())
    }
}

/// Batch processor for efficient parsing of multiple transactions
#[derive(Debug)]
pub struct BatchEventParser {
    /// Maximum number of transactions to process in a single batch
    max_batch_size: usize,
    /// Protocol-specific parsers indexed by protocol type
    parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>,
    /// SIMD pattern matcher for fast protocol detection
    pattern_matcher: SIMDPatternMatcher,
}

impl BatchEventParser {
    /// Add a parser for a specific protocol
    #[inline]
    pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>) {
        let protocol = parser.protocol_type();

        // Add to parsers map
        self.parsers.insert(protocol.clone(), parser);

        // Add discriminator pattern for fast detection
        match protocol {
            ProtocolType::Jupiter => {
                self.pattern_matcher.add_pattern(
                    vec![0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x8b],
                    protocol,
                ); // Route discriminator
            }
            ProtocolType::RaydiumAmmV4 => {
                self.pattern_matcher.add_pattern(vec![0x09], protocol); // SwapBaseIn discriminator
            }
            ProtocolType::OrcaWhirlpool
            | ProtocolType::MeteoraDlmm
            | ProtocolType::MarginFi
            | ProtocolType::Bonk
            | ProtocolType::PumpSwap
            | ProtocolType::RaydiumAmm
            | ProtocolType::RaydiumClmm
            | ProtocolType::RaydiumCpmm
            | ProtocolType::Raydium
            | ProtocolType::Serum
            | ProtocolType::Other(_) => {
                // Add default pattern
                self.pattern_matcher.add_pattern(vec![0x00], protocol);
            }
        }
    }

    /// Get statistics about the batch parser
    #[must_use]
    #[inline]
    pub fn get_stats(&self) -> BatchParserStats {
        BatchParserStats {
            max_batch_size: self.max_batch_size,
            pattern_count: 0, // patterns field is private, can't access directly
            registered_parsers: self.parsers.len(),
        }
    }

    /// Create a new batch parser
    #[must_use]
    #[inline]
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            parsers: HashMap::default(),
            pattern_matcher: SIMDPatternMatcher::default(),
        }
    }

    /// Parse a batch of transaction data
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the batch size exceeds the maximum or parsing fails.
    #[inline]
    pub fn parse_batch<'data>(
        &self,
        batch: &'data [&'data [u8]],
        metadatas: Vec<SolanaEventMetadata>,
    ) -> Result<Vec<ZeroCopyEvent<'data>>, ParseError> {
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
}

/// Statistics for batch parser performance monitoring
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BatchParserStats {
    /// Maximum batch size configured
    pub max_batch_size: usize,
    /// Number of discriminator patterns registered
    pub pattern_count: usize,
    /// Number of registered protocol parsers
    pub registered_parsers: usize,
}

/// Connection pool for RPC calls during parsing
pub struct RpcConnectionPool {
    /// Pool of shared RPC client instances
    clients: Vec<Arc<RpcClient>>,
    /// Current index for round-robin client selection
    current: AtomicUsize,
}

impl Debug for RpcConnectionPool {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RpcConnectionPool")
            .field("clients_count", &self.clients.len())
            .field("current", &self.current)
            .finish()
    }
}

impl RpcConnectionPool {
    /// Get the next client in round-robin fashion
    ///
    /// # Errors
    ///
    /// Returns `ParseError::NoRpcClientsAvailable` if no clients are configured.
    ///
    /// # Panics
    ///
    /// Panics if the modulo index is somehow out of bounds. This should be impossible
    /// due to the modulo operation with the vector length.
    #[inline]
    pub fn get_client(&self) -> Result<Arc<RpcClient>, ParseError> {
        if self.clients.is_empty() {
            return Err(ParseError::NoRpcClientsAvailable);
        }

        let index = self
            .current
            .fetch_add(1, Ordering::Relaxed)
            .checked_rem(self.clients.len())
            .unwrap_or(0);
        self.clients
            .get(index)
            .map(Arc::<RpcClient>::clone)
            .ok_or(ParseError::NoRpcClientsAvailable)
    }

    /// Create a new connection pool
    #[must_use]
    #[inline]
    pub fn new(urls: Vec<String>) -> Self {
        let clients = urls
            .into_iter()
            .map(|url| Arc::new(RpcClient::new(url)))
            .collect();

        Self {
            clients,
            current: AtomicUsize::default(),
        }
    }

    /// Get pool size
    #[inline]
    pub const fn size(&self) -> usize {
        self.clients.len()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_deserializer() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut deserializer = CustomDeserializer::new(&data);

        assert_eq!(
            deserializer
                .read_u8()
                .expect("Failed to read u8 from test data"),
            0x01
        );
        assert_eq!(
            deserializer
                .read_u32_le()
                .expect("Failed to read u32 from test data"),
            0x0504_0302
        );
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
        match result.expect_err("Expected InsufficientData error") {
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
        match result.expect_err("Expected InsufficientData error for u32 read") {
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
        match result.expect_err("Expected InsufficientData error for u64 read") {
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

        let result = deserializer
            .read_u64_le()
            .expect("Failed to read u64 from test data");
        assert_eq!(result, 0x0807_0605_0403_0201);
        assert_eq!(deserializer.position(), 8);
        assert_eq!(deserializer.remaining(), 0);
    }

    #[test]
    fn test_custom_deserializer_read_pubkey_insufficient_data() {
        let data = vec![0u8; 16]; // Only 16 bytes, need 32
        let mut deserializer = CustomDeserializer::new(&data);

        let result = deserializer.read_pubkey();
        assert!(result.is_err());
        match result.expect_err("Expected InsufficientData error for pubkey read") {
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
        match result.expect_err("Expected InsufficientData error for bytes read") {
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

        let result = deserializer
            .read_bytes(3)
            .expect("Failed to read 3 bytes from test data");
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
        match result.expect_err("Expected InsufficientData error for skip operation") {
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

        let _ = deserializer
            .read_u8()
            .expect("Failed to read u8 for skip test");
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
        let match_results = matcher.find_matches(&data);

        assert_eq!(match_results.len(), 3); // Two [0x01, 0x02] matches and one [0x03] match
        assert_eq!(
            *match_results.first().expect("Expected first match"),
            (0, ProtocolType::RaydiumAmmV4)
        );
        assert_eq!(
            *match_results.get(1).expect("Expected second match"),
            (2, ProtocolType::Jupiter)
        );
        assert_eq!(
            *match_results.get(2).expect("Expected third match"),
            (3, ProtocolType::RaydiumAmmV4)
        );
    }

    #[test]
    fn test_simd_pattern_matcher_data_too_short() {
        let mut matcher = SIMDPatternMatcher::default();
        matcher.add_pattern(vec![0x01, 0x02, 0x03], ProtocolType::RaydiumAmmV4);

        let data = vec![0x01, 0x02]; // Too short for pattern
        let match_results = matcher.find_matches(&data);

        assert!(match_results.is_empty());
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

        let mut parser = BatchEventParser::new(100);

        parser.add_parser(Arc::new(MockParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_jupiter() {
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

        let mut parser = BatchEventParser::new(100);

        parser.add_parser(Arc::new(MockJupiterParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_add_parser_other_protocol() {
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

        let mut parser = BatchEventParser::new(100);

        parser.add_parser(Arc::new(MockOtherParser));

        let stats = parser.get_stats();
        assert_eq!(stats.registered_parsers, 1);
        assert_eq!(stats.pattern_count, 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_exceeds_max_size() {
        let parser = BatchEventParser::new(2);

        let data1 = [0x01, 0x02];
        let data2 = [0x03, 0x04];
        let data3 = [0x05, 0x06];
        let batch = vec![&data1[..], &data2[..], &data3[..]]; // 3 items, max is 2

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata.clone(), metadata.clone(), metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_err());
        match result.expect_err("Expected InvalidInstructionData error for batch size") {
            ParseError::InvalidInstructionData(msg) => {
                assert!(msg.contains("Batch size 3 exceeds maximum 2"));
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    #[test]
    fn test_batch_event_parser_parse_batch_success() {
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

        let mut parser = BatchEventParser::new(10);

        parser.add_parser(Arc::new(MockSuccessParser));

        let data1 = [0x09, 0x02]; // Starts with Raydium discriminator
        let batch = vec![&data1[..]];

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_batch_event_parser_parse_batch_no_matching_protocol() {
        let parser = BatchEventParser::new(10);

        let data1 = [0xFF, 0xFF]; // No matching discriminator
        let batch = vec![&data1[..]];

        let metadata = create_solana_metadata(
            String::default(),
            String::default(),
            0,
            0,
            ProtocolType::default(),
            EventType::default(),
            Pubkey::default(),
            String::default(),
            0,
        );
        let metadatas = vec![metadata];

        let result = parser.parse_batch(&batch, metadatas);
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
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
        let _client1 = pool.get_client().expect("Failed to get RPC client 1");
        let _client2 = pool.get_client().expect("Failed to get RPC client 2");
        let _client3 = pool.get_client().expect("Failed to get RPC client 3");

        // Should work without panicking
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_rpc_connection_pool_empty_error() {
        let pool = RpcConnectionPool::new(vec![]);
        let result = pool.get_client();
        assert!(matches!(result, Err(ParseError::NoRpcClientsAvailable)));
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
        assert_eq!(format!("{error1}"), "Invalid instruction data: test error");

        let error2 = ParseError::InsufficientData {
            expected: 10,
            actual: 5,
        };
        assert_eq!(
            format!("{error2}"),
            "Insufficient data length: expected 10, got 5"
        );

        let error3 = ParseError::UnknownDiscriminator {
            discriminator: vec![0x01, 0x02],
        };
        assert_eq!(format!("{error3}"), "Unknown discriminator: [1, 2]");

        let error4 = ParseError::MemoryMapError("mmap failed".to_string());
        assert_eq!(format!("{error4}"), "Memory map error: mmap failed");
    }

    // Test BatchParserStats
    #[test]
    fn test_batch_parser_stats_clone() {
        let stats = BatchParserStats {
            registered_parsers: 5,
            max_batch_size: 100,
            pattern_count: 3,
        };

        let cloned_stats = stats;
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
        assert_eq!(
            result.expect("Failed to read u32 at exact boundary"),
            0x0403_0201
        );
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
        assert_eq!(
            result.expect("Failed to read u64 at exact boundary"),
            0x0807_0605_0403_0201
        );
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
        assert_eq!(result.expect("Failed to read zero-length bytes").len(), 0);
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
        assert_eq!(
            deserializer
                .read_u8()
                .expect("Failed to read u8 in mixed operations"),
            0x01
        );
        assert_eq!(deserializer.position(), 1);

        // Skip 2 bytes
        deserializer
            .skip(2)
            .expect("Failed to skip 2 bytes in mixed operations");
        assert_eq!(deserializer.position(), 3);

        // Read 3 bytes
        let bytes = deserializer
            .read_bytes(3)
            .expect("Failed to read 3 bytes in mixed operations");
        assert_eq!(bytes, &[0x04, 0x05, 0x06]);
        assert_eq!(deserializer.position(), 6);

        // Read u32
        assert_eq!(
            deserializer
                .read_u32_le()
                .expect("Failed to read final u32 in mixed operations"),
            0x0A09_0807
        );
        assert_eq!(deserializer.position(), 10);
        assert_eq!(deserializer.remaining(), 0);
    }

    // Test MemoryMappedParser functionality
    #[test]
    fn test_memory_mapped_parser_from_file_nonexistent() {
        let result = MemoryMappedParser::from_file("/nonexistent/file/path.dat");
        assert!(result.is_err());

        // Should be a file not found error wrapped in ParseError
        match result.expect_err("Expected error for nonexistent file") {
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
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"Hello, World! This is test data for memory mapping.";
        temp_file
            .write_all(test_data)
            .expect("Failed to write test data");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        // Test valid slice
        let slice = parser.data_slice(0, 5);
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get Hello slice"), b"Hello");

        // Test slice in middle
        let slice = parser.data_slice(7, 5);
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get World slice"), b"World");

        // Test entire data
        let slice = parser.data_slice(0, test_data.len());
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get entire data slice"), test_data);
    }

    #[test]
    fn test_memory_mapped_parser_data_slice_invalid_range() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"Small data";
        temp_file
            .write_all(test_data)
            .expect("Failed to write test data to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

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
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"Test data for size checking";
        temp_file
            .write_all(test_data)
            .expect("Failed to write test data to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        assert_eq!(parser.size(), test_data.len());
    }

    #[test]
    fn test_memory_mapped_parser_add_parser() {
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

        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let mut parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        parser.add_parser(Arc::new(TestParser));

        // Verify parser was added (we can't directly access the HashMap, but we can test parse_all doesn't panic)
        let result = parser.parse_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_with_parser() {
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
                !data.is_empty()
            }

            fn protocol_type(&self) -> ProtocolType {
                ProtocolType::Jupiter
            }
        }

        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"test data for parsing";
        temp_file
            .write_all(test_data)
            .expect("Failed to write test data to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let mut parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        parser.add_parser(Arc::new(MockEventParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events
                .first()
                .expect("Expected at least one event")
                .raw_data(),
            test_data
        );
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_no_parsers() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_parser_cannot_parse() {
        use std::io::Write;

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

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let mut parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        parser.add_parser(Arc::new(CannotParseParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_memory_mapped_parser_parse_all_parser_error() {
        use std::io::Write;

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

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let mut parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        parser.add_parser(Arc::new(ErrorParser));

        let result = parser.parse_all();
        assert!(result.is_err());
        match result.expect_err("Expected parsing error from error parser") {
            ParseError::InvalidInstructionData(msg) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected InvalidInstructionData error"),
        }
    }

    // Test ParseError from conversion
    #[test]
    fn test_parse_error_from_io_error() {
        use std::io::{Error as IoError, ErrorKind};

        let io_error = IoError::new(ErrorKind::NotFound, "File not found");
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
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        // Zero length slice should be valid
        let slice = parser.data_slice(0, 0);
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get zero-length slice").len(), 0);

        // Zero length slice at end should also be valid
        let slice = parser.data_slice(4, 0);
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get zero-length slice").len(), 0);
    }

    // Test boundary condition where offset + len equals exactly the data length
    #[test]
    fn test_memory_mapped_parser_data_slice_exact_boundary() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"test";
        temp_file
            .write_all(test_data)
            .expect("Failed to write test data to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        // Exact boundary should work
        let slice = parser.data_slice(0, test_data.len());
        assert!(slice.is_some());
        assert_eq!(slice.expect("Failed to get test data slice"), test_data);

        // One byte past should fail
        let slice = parser.data_slice(0, test_data.len() + 1);
        assert!(slice.is_none());
    }

    // Test ZeroCopyEvent field access in parse_all
    #[test]
    fn test_memory_mapped_parser_parse_all_event_modification() {
        use std::io::Write;

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

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"test")
            .expect("Failed to write 'test' to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let mut parser = MemoryMappedParser::from_file(
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert temp file path to string"),
        )
        .expect("Failed to create memory mapped parser");

        parser.add_parser(Arc::new(MultiEventParser));

        let result = parser.parse_all();
        assert!(result.is_ok());
        let events = result.expect("Failed to parse batch events successfully");
        assert_eq!(events.len(), 2);
    }
}
