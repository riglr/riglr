//! Common log-based event parsing utilities
//!
//! This module provides shared parsing logic for log-based protocols
//! that emit events through inner instructions (program logs).
//! 
//! Used primarily by protocols like Raydium CPMM that emit structured
//! event data through program logs rather than instruction parameters.

use borsh::BorshDeserialize;
use hex;

use crate::events::common::{EventMetadata, utils::discriminator_matches};
use crate::events::core::traits::UnifiedEvent;

/// Common log event parsing utilities
pub struct LogEventParser;

impl LogEventParser {
    /// Parse hex-encoded log data with discriminator validation
    pub fn parse_log_with_discriminator(
        log_data: &str,
        expected_discriminator: &str,
        skip_bytes: usize,
    ) -> Option<Vec<u8>> {
        if !discriminator_matches(log_data, expected_discriminator) {
            return None;
        }
        
        // Remove discriminator prefix and convert from hex
        if log_data.starts_with("0x") {
            let hex_data = &log_data[2..]; // Remove "0x" prefix
            if let Ok(decoded) = hex::decode(hex_data) {
                if decoded.len() > skip_bytes {
                    return Some(decoded[skip_bytes..].to_vec());
                }
            }
        }
        None
    }
    
    /// Parse base58-encoded log data (alternative encoding format)
    pub fn parse_base58_log(log_data: &str, skip_bytes: usize) -> Option<Vec<u8>> {
        if let Ok(decoded) = bs58::decode(log_data).into_vec() {
            if decoded.len() > skip_bytes {
                return Some(decoded[skip_bytes..].to_vec());
            }
        }
        None
    }
    
    /// Extract discriminator from log data
    pub fn extract_discriminator(log_data: &str, discriminator_length: usize) -> Option<String> {
        if log_data.starts_with("0x") && log_data.len() >= 2 + discriminator_length * 2 {
            Some(log_data[..2 + discriminator_length * 2].to_string())
        } else {
            None
        }
    }
}

/// Borsh deserialization for log events
pub struct LogBorshParser;

impl LogBorshParser {
    /// Parse borsh-serialized log event
    pub fn parse_borsh_log<T>(data: &[u8]) -> Option<T>
    where
        T: BorshDeserialize,
    {
        T::try_from_slice(data).ok()
    }
    
    /// Parse with metadata integration and ID generation
    /// This is a utility for protocols that use borsh serialization in logs
    pub fn parse_borsh_with_metadata<T>(
        log_data: &str,
        expected_discriminator: &str,
        skip_bytes: usize,
    ) -> Option<T>
    where
        T: BorshDeserialize,
    {
        let data = LogEventParser::parse_log_with_discriminator(
            log_data, 
            expected_discriminator, 
            skip_bytes
        )?;
        
        Self::parse_borsh_log::<T>(&data)
    }
}

/// Discriminator management for log events
pub struct LogDiscriminatorManager;

impl LogDiscriminatorManager {
    /// Standard discriminator lengths for different log formats
    pub const STANDARD_DISCRIMINATOR_LENGTH: usize = 8;
    pub const EXTENDED_DISCRIMINATOR_LENGTH: usize = 16;
    
    /// Check if log matches any of the provided discriminators
    pub fn matches_any_discriminator(log_data: &str, discriminators: &[&str]) -> Option<String> {
        for &discriminator in discriminators {
            if discriminator_matches(log_data, discriminator) {
                return Some(discriminator.to_string());
            }
        }
        None
    }
    
    /// Validate discriminator format
    pub fn is_valid_discriminator(discriminator: &str) -> bool {
        discriminator.starts_with("0x") 
            && discriminator.len() >= 4  // At least 2 hex bytes
            && discriminator.len() % 2 == 0  // Even length for hex pairs
            && discriminator[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Common patterns for log event ID generation
pub struct LogEventIdGenerator;

impl LogEventIdGenerator {
    /// Generate ID for log-based swap events
    pub fn log_swap_id(signature: &str, pool_key: &str, direction: &str) -> String {
        format!("{}-{}-{}-log", signature, pool_key, direction)
    }
    
    /// Generate ID for log-based pool events
    pub fn log_pool_id(signature: &str, pool_state: &str, operation: &str) -> String {
        format!("{}-{}-{}-log", signature, pool_state, operation)
    }
    
    /// Generate ID for log-based liquidity events
    pub fn log_liquidity_id(signature: &str, position_id: &str, operation: &str) -> String {
        format!("{}-{}-{}-log", signature, position_id, operation)
    }
}

/// Event merging utilities for combining instruction and log data
pub struct LogEventMerger;

impl LogEventMerger {
    /// Check if two events should be merged based on their IDs and timing
    pub fn should_merge_events(
        instruction_event_id: &str,
        log_event_id: &str,
        instruction_index: &str,
        log_index: &str,
    ) -> bool {
        // Extract base IDs without suffixes
        let instruction_base = instruction_event_id.replace("-instruction", "");
        let log_base = log_event_id.replace("-log", "");
        
        if instruction_base != log_base {
            return false;
        }
        
        // Check if log event follows instruction event in the same transaction
        if let (Ok(inst_idx), Ok(log_idx)) = (
            instruction_index.parse::<u32>(),
            log_index.parse::<u32>()
        ) {
            log_idx > inst_idx
        } else {
            // Handle nested indices like "1.2"
            LogEventMerger::compare_nested_indices(instruction_index, log_index)
        }
    }
    
    /// Compare nested instruction indices (e.g., "1.2" vs "1.3")
    fn compare_nested_indices(index1: &str, index2: &str) -> bool {
        let parts1: Vec<&str> = index1.split('.').collect();
        let parts2: Vec<&str> = index2.split('.').collect();
        
        if parts1.len() != parts2.len() {
            return false;
        }
        
        // Check if they're in the same instruction group
        if !parts1.is_empty() && !parts2.is_empty() && parts1[0] == parts2[0] {
            if parts1.len() >= 2 && parts2.len() >= 2 {
                if let (Ok(idx1), Ok(idx2)) = (parts1[1].parse::<u32>(), parts2[1].parse::<u32>()) {
                    return idx2 > idx1;
                }
            }
        }
        
        false
    }
}

/// Log format detection and parsing strategy selection
pub struct LogFormatDetector;

impl LogFormatDetector {
    /// Detect log format based on data structure
    pub fn detect_format(log_data: &str) -> LogFormat {
        if log_data.starts_with("0x") {
            if LogDiscriminatorManager::is_valid_discriminator(log_data) {
                LogFormat::HexWithDiscriminator
            } else {
                LogFormat::HexRaw
            }
        } else if log_data.chars().all(|c| c.is_ascii_alphanumeric()) {
            LogFormat::Base58
        } else {
            LogFormat::Unknown
        }
    }
    
    /// Select appropriate parsing strategy
    pub fn select_parser_strategy(format: LogFormat) -> ParsingStrategy {
        match format {
            LogFormat::HexWithDiscriminator => ParsingStrategy::HexDiscriminator,
            LogFormat::HexRaw => ParsingStrategy::HexRaw,
            LogFormat::Base58 => ParsingStrategy::Base58,
            LogFormat::Unknown => ParsingStrategy::Fallback,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    HexWithDiscriminator,
    HexRaw,
    Base58,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsingStrategy {
    HexDiscriminator,
    HexRaw,
    Base58,
    Fallback,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discriminator_validation() {
        assert!(LogDiscriminatorManager::is_valid_discriminator("0x1234567890abcdef"));
        assert!(LogDiscriminatorManager::is_valid_discriminator("0x12"));
        assert!(!LogDiscriminatorManager::is_valid_discriminator("1234567890abcdef")); // No 0x prefix
        assert!(!LogDiscriminatorManager::is_valid_discriminator("0x123")); // Odd length
        assert!(!LogDiscriminatorManager::is_valid_discriminator("0x12xyz8")); // Invalid hex
    }
    
    #[test]
    fn test_discriminator_matching() {
        let log_data = "0x1234567890abcdef00112233";
        let discriminators = ["0x1234567890abcdef", "0x9876543210fedcba"];
        
        let matched = LogDiscriminatorManager::matches_any_discriminator(log_data, &discriminators);
        assert_eq!(matched, Some("0x1234567890abcdef".to_string()));
    }
    
    #[test]
    fn test_log_format_detection() {
        assert_eq!(LogFormatDetector::detect_format("0x1234567890abcdef"), LogFormat::HexWithDiscriminator);
        assert_eq!(LogFormatDetector::detect_format("0xinvalidhex"), LogFormat::HexRaw);
        assert_eq!(LogFormatDetector::detect_format("base58string123"), LogFormat::Base58);
        assert_eq!(LogFormatDetector::detect_format("invalid@#$"), LogFormat::Unknown);
    }
    
    #[test]
    fn test_event_merging_logic() {
        // Test simple numeric indices
        assert!(LogEventMerger::should_merge_events(
            "sig-pool-swap-instruction",
            "sig-pool-swap-log", 
            "1",
            "2"
        ));
        
        // Test nested indices
        assert!(LogEventMerger::should_merge_events(
            "sig-pool-swap-instruction",
            "sig-pool-swap-log",
            "1.1",
            "1.2"
        ));
        
        // Test non-matching base IDs
        assert!(!LogEventMerger::should_merge_events(
            "sig-pool1-swap-instruction",
            "sig-pool2-swap-log",
            "1",
            "2"
        ));
    }
    
    #[test]
    fn test_hex_log_parsing() {
        let log_data = "0x1234567890abcdef00112233445566778899aabb";
        let expected_discriminator = "0x1234567890abcdef";
        
        let parsed = LogEventParser::parse_log_with_discriminator(log_data, expected_discriminator, 0);
        assert!(parsed.is_some());
        
        let data = parsed.unwrap();
        assert_eq!(data, hex::decode("1234567890abcdef00112233445566778899aabb").unwrap());
    }
}